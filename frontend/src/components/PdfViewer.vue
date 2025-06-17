<template lang="pug">
div.viewer
	.scroll(@scroll="scroll_pdfs")
		.pdf-container
			template(v-for="(pdf, index) in pages")
				.pdf-page(:ref="set_item_ref" :id="`page_container_${index+1}`")
					transition(name="fade" mode="out-in")
						n-spin.loader-style(v-if="!page_loaded(pdf)" size="large")
							template(#description) Загрузка страницы {{ index+1 }}
						img(v-else :src="pdf")
					.splitter
	.footer-style
		n-input-number(v-model:value="scroll_to_page_value" :max="pages_count" :min="1" size="large" @keydown.enter="scroll_to_page(scroll_to_page_value)")
			template(#suffix)
				div(style="background-color: #3d636b") {{ current_page_number }}/{{pages_count}}
</template>

<script lang="ts">
// :class="{'bluring': in_progress, 'unbluring': !in_progress}"
import { ref, type Component, defineAsyncComponent, watch, inject, onMounted, onUnmounted, computed, onUpdated, nextTick } from 'vue';
import { NNotificationProvider, NInputNumber, NScrollbar, NLoadingBarProvider, NButton, NIcon, NSkeleton, NTooltip, NDrawer, NDrawerContent, NPagination, NSpin, NProgress, useLoadingBar} from 'naive-ui';
import { Analytics, Document, Warning, Settings, FingerPrintSharp, PieChart, PulseSharp } from '@vicons/ionicons5';
//import { type Status } from 'naive-ui/es/progress/src/interface';
import { type Emitter, type Events } from '@/services/emitter';
import { http_sevice } from '@/services/http_service';
import { homer_ico } from '@/services/svg';
import { sleepNow } from '@/services/helpers';
import Loader from '@components/Loader.vue';
interface Props 
{
	doc_id: number
}
</script>

<script lang="ts" setup>

const props = defineProps<Props>();
const emitter = inject<Emitter<Events>>('emitter') as Emitter<Events>;
const pages = ref<string[]>([])
const is_open = ref(false);
const pages_count = ref(1);
const scroll_to_page_value = ref(1);
const current_document_id = ref(0);
const current_page_number = ref(1);
const page_loaded = (pdf: string) =>
{
    return pdf != ""
}
const item_refs = ref<Map<string, HTMLElement>>(new Map())
const set_item_ref = (el: HTMLElement|null) =>
{
	if(el)
	{
		item_refs.value.set(el.id, el)
	}
}
const scroll_container = ref<HTMLElement | null>(null);

//TODO сомневаюсь в скорости на больших массивах...
const scroll_pdfs = () =>
{
	const container = document.querySelector(".scroll");
	const container_rect = container?.getBoundingClientRect();
	if(container_rect)
	{
		let page_index = 1;
		item_refs.value.forEach(f=>
		{
			let rect = f.getBoundingClientRect();
			if(rect.top <= container_rect.bottom /2)
			{
				current_page_number.value = page_index;
				//page_number.value = `Страница: ${page_index}/${pages.value.length}` ;
				page_index++;
			}
		})
	}
}
const scroll_to_page = async (number: number) =>
{
  //await nextTick(); // Ждем обновления DOM
	console.log("try scrioll to ", number);
	const container = document.querySelector(".scroll");
	const element = item_refs.value.get(`page_container_${number}`);
	console.log(container, element);
	if (container && element) 
	{
		const container_rect = container.getBoundingClientRect();
		const element_rect = element.getBoundingClientRect();
		
		// Прокручиваем с учетом текущей позиции контейнера
		container.scrollTo({
		top: element_rect.top - container_rect.top + container.scrollTop,
		behavior: 'smooth'
		});
	}
}

const get_pdf = async (doc_id: number) =>
{
	is_open.value = true;
	console.log(doc_id)
	//чтобы не загружать заново для того же документа
	if(current_document_id.value != doc_id)
	{
		pages_count.value = 1;
		current_document_id.value = doc_id;
		pages_count.value = await http_sevice.get_pdf_pages_count(doc_id);
		pages.value = Array.from({ length: pages_count.value }, () => "");
		current_page_number.value = 1;
		//для начала получаем все страницы
		const all_pages = Array.from({ length: pages_count.value }, (_, i) => 1 + i);
		const reader_result = await http_sevice.get_pdf_images(doc_id, all_pages);
		if(reader_result.ok)
		{
			let buffer = new Uint8Array();
			let expected_size: number | null = null;
			let page = 1;
			const separator = new TextEncoder().encode("--IMAGE-BOUNDARY--");
			const separator_length = separator.length;
			while (true) 
			{
				const { done, value } = await reader_result.val.read();
				if (done) 
				{
					if (buffer.length > 0) 
					{
						console.warn('Unprocessed data remaining at end of stream');
					}
					break;
				}

				// Добавляем новые данные в буфер
				buffer = concat_uint8_arrays(buffer, value);
				
				// Обрабатываем все полные изображения в буфере
				while (true) 
				{
					// Если ожидаем размер изображения и буфер содержит достаточно данных
					if (expected_size === null && buffer.length >= 4) 
					{
						expected_size = new DataView(buffer.buffer, buffer.byteOffset).getUint32(0, true);
						buffer = buffer.slice(4); // Удаляем размер из буфера
					}
					
					// Если ожидаем данные изображения и буфер содержит достаточно данных
					if (expected_size !== null && buffer.length >= expected_size) 
					{
						const imageData = buffer.slice(0, expected_size);
						const blob = new Blob([imageData], { type: 'image/webp' });
						//await sleepNow(2000);
						pages.value[page - 1] = URL.createObjectURL(blob);
						page++;

						buffer = buffer.slice(expected_size);
						expected_size = null;
						//удаляем сепаратор
						if (buffer.length >= separator_length) 
						{
							const currentSeparator = buffer.slice(0, separator_length);
							if (compare_uint8_arrays(currentSeparator, separator)) 
							{
								buffer = buffer.slice(separator_length);
							} 
							else 
							{
								console.error('Invalid separator found');
							}
						}
						continue; // Проверяем, есть ли еще данные для обработки
					}

					break; // Не хватает данных для следующего изображения
				}
			}
		}
	}
}

// Вспомогательная функция для объединения Uint8Array
function concat_uint8_arrays(a: Uint8Array, b: Uint8Array): Uint8Array 
{
    const result = new Uint8Array(a.length + b.length);
    result.set(a);
    result.set(b, a.length);
    return result;
}

function compare_uint8_arrays(a: Uint8Array, b: Uint8Array): boolean 
{
    if (a.length !== b.length) return false;
    for (let i = 0; i < a.length; i++) 
	{
        if (a[i] !== b[i]) return false;
    }
    return true;
}

watch(() => props.doc_id, (n) =>
{
	get_pdf(n)
}, {immediate: true})

onMounted(() =>
{
    //emitter.on('open_pdf', get_pdf);
})

onUnmounted(() => 
{
	pages.value.forEach(f=>
	{
		URL.revokeObjectURL(f);
	})
	//emitter.off('open_pdf', get_pdf)
})

const get_element_when_hover = () =>
{
	var n = document.querySelector(":hover");
	var nn;
	while(n)
	{
		nn = n;
		n = nn.querySelector(":hover")
	}
	return nn;
}

</script>
    
<style lang="scss">
.splitter
{
	position:relative;
	width: inherit;
	height: 1px;
	background-color: rgba(94, 90, 90, 0.315);
}
.scroll
{
	display: block;
  	overflow-y: auto;
	scroll-behavior: smooth;
}
.viewer
{
	height: 100vh;
	max-height: calc(100vh - 65px);
}
.pdf-container
{
	height: 100%;
	min-height: 800px;
	min-width: 100%;
	width: inherit;
	display: flex;
    flex-direction: column;
}
.pdf-page
{
    min-height: inherit;
    min-width: inherit;
	background-color: white;
}
.loader-style
{
	min-height: inherit;
	min-width: inherit;
	background-color: rgb(153, 153, 153);
}

.file-drawler
{
    width: 650px;
    min-width: 620px;
    overflow-x: hidden;
    overflow-y: hidden;
    .n-base-selection .n-base-selection-label
    {
    	height: initial !important;
    }
}

.footer-style
{
   margin-top: 5px;
}
.page-number-style
{
	flex-grow: 2;
}

.fade-enter-active,
.fade-leave-active
{
	transition: opacity 0.3s ease;
}
.fade-enter-from,
.fade-leave-to
{
	opacity: 0;
}
.fade-enter-to,
.fade-leave-from
{
	opacity: 1;
}

.bluring
{
    animation:  bluring-animate 0.2s;
    filter: blur(2.5px);
    -webkit-filter: blur(2.5px);
    //transform: translateY(120%);
    //-webkit-transform: translateY(120%);
}
.unbluring
{
    animation:  unbluring-animate 0.1s;
    filter: blur(0px);
    -webkit-filter: blur(0px);
    //transform: translateY(0%);
    //-webkit-transform: translateY(0%);
}

@keyframes bluring-animate
{
    0% 
    {
    filter: blur(0.5px);
    //transform: translateY(0%);
    //-webkit-transform: translateY(0%);
    -webkit-filter: blur(0.5px);
    }
    20%
    {
    filter: blur(1px);
    //transform: translateY(30%);
    //-webkit-transform: translateY(30%);
    -webkit-filter: blur(1px);
    }
    40%
    {
    filter: blur(1.5px);
    //transform: translateY(30%);
    //-webkit-transform: translateY(30%);
    -webkit-filter: blur(1.5px);
    }
    70% 
    {
    filter: blur(2px);
    //transform: translateY(70%);
    //-webkit-transform: translateY(70%);
    -webkit-filter: blur(2px);
    }
    100% 
    {
    filter: blur(2.5px);
    //transform: translateY(100%);
    //-webkit-transform: translateY(100%);
    -webkit-filter: blur(2.5px);
    }
}
@keyframes y-on-animate
{
    0% 
    {
    transform: translateY(0%);
    -webkit-transform: translateY(0%);
    }
    20%
    {
    transform: translateY(20%);
    -webkit-transform: translateY(20%);
    }
    40%
    {
    transform: translateY(40%);
    -webkit-transform: translateY(40%);
    }
    70% 
    {
    transform: translateY(70%);
    -webkit-transform: translateY(70%);
    }
    100% 
    {
    transform: translateY(100%);
    -webkit-transform: translateY(100%);
    }
}
@keyframes y-off-animate
{
    0% 
    {
    transform: translateY(-100%);
    -webkit-transform: translateY(-100%);
    }
    20%
    {
    transform: translateY(-80%);
    -webkit-transform: translateY(-80%);
    }
    40%
    {
    transform: translateY(-60%);
    -webkit-transform: translateY(-60%);
    }
    70% 
    {
    transform: translateY(-30%);
    -webkit-transform: translateY(-30%);
    }
    100% 
    {
    transform: translateY(0%);
    -webkit-transform: translateY(0%);
    }
}

@keyframes unbluring-animate
{
    0% 
    {
    filter: blur(2px);
    //transform: translateY(-100%);
    //-webkit-transform: translateY(-100%);
    -webkit-filter: blur(2px);
    }
    20% 
    {
    filter: blur(1.5px);
    //transform: translateY(-70%);
    //-webkit-transform: translateY(-70%);
    -webkit-filter: blur(1.5px);
    }
    40% 
    {
    filter: blur(1px);
    //transform: translateY(-70%);
    //-webkit-transform: translateY(-70%);
    -webkit-filter: blur(1px);
    }
    70% 
    {
    filter: blur(0.5px);
    //transform: translateY(-30%);
    //-webkit-transform: translateY(-30%);
    -webkit-filter: blur(0.5px);
    }
    100% 
    {
    filter: blur(0px);
    //transform: translateY(0%);
    //-webkit-transform: translateY(0%);
    -webkit-filter: blur(0px);
    }
}
</style>
            