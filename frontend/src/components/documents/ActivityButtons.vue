<template lang="pug">
div.buttons-panel
    div Загруженные пакеты
    n-select(:options="options" v-model:value="selected_item" @update:value="on_selected")
</template>
        
<script lang="ts">
import { defineComponent, ref, onMounted, watch, computed} from 'vue'
import {NSpace, NButton, NDataTable, NIcon, NTooltip, NDivider, NSelect, NDatePicker} from 'naive-ui'

import { type Documents, type DateUserInfo, type Document} from '@/types/document';
import {type Redaction } from '@/types/document'
import UserIcon from '@/components/UserIcon.vue'
import { InformationSquare } from '@vicons/carbon'
import { date_str } from '@/services/helpers'
import { DateFormat, DateTime } from '@/services/date'
import { http_sevice } from '@/services/http_service';
import { match } from 'ts-pattern';
import { type Packets, type Packet} from '@/types/packet';
interface SelectItemInterface
{
    label: string,
    value: string,
    //packet: Packet
}
</script>

<script lang="ts" setup>
const props = defineProps<{
    documents: Documents,
    count: number,
    offset: number,
    limit: number,
    is_loading: boolean
}>()

const emits = defineEmits<{
    (e: 'update:documents', docs: Documents): void,
    (e: 'update:count', cnt: number): void,
    (e: 'update:is_loading', cnt: boolean): void,
    (e: 'update:offset', cnt: number): void
}>()
const packets = ref<Packets>([]);
const selected_item = ref<string|null>(null);
const options = computed(() => packets.value.map(p=>
{
    const date = p.created.to_string(DateFormat.DotDate);
    const time = p.created.to_string(DateFormat.Time);
    return {
          label: `${date} ${time}`,
          value: p.id
        } as SelectItemInterface
}))
const on_selected = async (value: string, option: SelectItemInterface) =>
{
    await load_docs();
    update_value(value)
}
const update_value = (v: string) =>
{
    localStorage.setItem('selected_packet', v)
}

const load_docs_handler = async () =>
{
    emits('update:offset', 0);
    await load_docs();
}
const load_docs = async () =>
{
    if(selected_item.value)
    {
        emits('update:is_loading', true);
        const count = await http_sevice.get_packet_count(selected_item.value);
        emits('update:count', count);
        const docs = await http_sevice.get_packet_documents(selected_item.value, props.limit, props.offset);
        emits('update:documents', docs);
        emits('update:is_loading', false);
    }
}

onMounted(async ()=> 
{
    const p = await http_sevice.get_packets();
    p.sort((a, b) => a.created.gt(b.created) ? 0 : 1)
    packets.value = p
    const selected = localStorage.getItem('selected_packet');
    if(selected)
    {
        selected_item.value = selected;
        await load_docs()
    }
})
//перелистываем страницы
watch(()=> props.offset, async (n, o) => 
{
    if(n != o || (n == 0 && o == 0))
    {
        await load_docs();
    }
})

</script>
    
<style lang="scss" scoped>
.buttons-panel
{
    display: flex;
    flex-direction: row;
    gap: 3px;
    flex-wrap: nowrap;
    width: 800px;
}

</style>