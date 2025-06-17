<template lang="pug">
n-scrollbar(style="max-height: 130px")
    .priv
        n-checkbox(v-for="p in all_privileges" :disabled="props.disabled" @update:checked="checked" v-model:checked="p.checked" :key="p.name") {{p.description}}
</template>
        
<script lang="ts">
import {UserAvatarFilledAlt} from '@vicons/carbon'
import { defineComponent, ref, onMounted, watch, computed} from 'vue'
import {NTooltip, NAvatar, NIcon, NCheckbox, NScrollbar} from 'naive-ui'
import { type Document } from "@/types/document"
import useUser from '@composables/useUser'
import { Error } from '@vicons/carbon'
import { base64_to_uint8_array, sleepNow } from '@/services/helpers'
import { type Privilegy, privileges } from '@/types/privilegy';
import { match } from 'ts-pattern'
</script>

<script lang="ts" setup>
interface Props 
{
    value?: Privilegy[],
    disabled: boolean
}
interface Checked
{
    name: Privilegy,
    description: string,
    checked: boolean
}
const props = defineProps<Props>();
const emits = defineEmits<{
    (e: 'update:value', profile: Privilegy[]): void
}>()
const all_privileges = ref<Checked[]>([]);
const {get_privilegy_description} = useUser();
watch(() => props.value, (n) =>
{
    all_privileges.value = [];
    privileges.forEach(f=>
    {
        let checked = n?.includes(f);
        const descriprion = get_privilegy_description(f);
        all_privileges.value.push({name: f, description: descriprion, checked: checked ?? false})
    })
}, {immediate: true})
const checked = async (v: string) => 
{
    await sleepNow(200); //hmm событие вызывается до того как сработает привязка к 
    // all_privileges это конечно решает вопрос но похоже на костыль
    const priv = all_privileges.value.filter(f=>f.checked == true).map(m=>m.name);
    emits('update:value', priv);
}
// const sorted_privileges = computed(() => 
// {
//   return [...all_privileges.value].sort((a, b) => 
//     Number(b.checked) - Number(a.checked))
// });
</script>
    
<style lang="scss" scoped>
.mr-4
{
    margin-right: 4px;
}
.priv
{
    display: flex;
    flex-direction: column;
}
</style>