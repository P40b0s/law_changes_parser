<template lang="pug">
n-tooltip(trigger="hover" @update:show="on_open")
    template(#trigger)
        n-icon(size="30" :color="icon_color" :component="InformationSquareFilled")
    div.statuses
    .status-icon
        n-icon(size="35" :color="icon_color" :component="InformationSquareFilled")
        .text  {{information}}
    .status(v-for="r in times")
        user-icon.user-icon(:user_id="r[2].user_id", :visible_if_not_found="true" :size="50")
        .text-field
            span.text {{r[0] + r[1]}}
            span.text {{r[3]}}
            
</template>
        
<script lang="ts">
import { defineComponent, ref, onMounted} from 'vue'
import {NSpace, NButton, NDataTable, NIcon, NTooltip, NDivider} from 'naive-ui'

import { type DateUserInfo, type Document } from "@/types/document"
import {type Redaction } from '@/types/document'
import UserIcon from '@components/UserIcon.vue'
import { InformationSquareFilled } from '@vicons/carbon'
import { date_str } from '@services/helpers'
import { DateFormat, DateTime } from '@services/date'
import { http_sevice } from '@/services/http_service'
</script>

<script lang="ts" setup>
const props = defineProps<{
    redaction: Redaction,
    checked: DateUserInfo | null
}>()
const icon_color = ref('#59d51a');
const information = ref("Документ готов к выгрузке");
const get_fio = async (user_id: number) =>
{
    const user = await http_sevice.get_user(user_id);
    if(user)
    {
        return `${user.surname} ${user.first_name} ${user.second_name}`
    }
    else return "Неизвестно"
}
const times = ref<[string, string, DateUserInfo, string][]>([]);
const on_open = async (v: boolean) =>
{
    if(v)
        times.value.forEach(async f=>
        {
            f[3] = await get_fio(f[2].user_id)
        })
}
onMounted(async ()=>
{
    if(props.redaction.redaction_create)
    times.value.push(["Создан: ", props.redaction.redaction_create.date.to_string(DateFormat.DateTime), props.redaction.redaction_create, ""])
    if(props.redaction.redaction_update)
        times.value.push(["Обновлен: ", props.redaction.redaction_update.date.to_string(DateFormat.DateTime), props.redaction.redaction_update, ""])
    if(props.redaction.redaction_ready)
    {
        times.value.push(["Готов: ", props.redaction.redaction_ready.date.to_string(DateFormat.DateTime), props.redaction.redaction_ready, ""])
    }
    else
    {
        icon_color.value = '#d51a1a'
        information.value = "Необходимо поставить отметку `завершен` в приложении `Комплекс`";
    }  
    if(props.checked)
    {
        times.value.push(["Проверен: ", props.checked.date.to_string(DateFormat.DateTime), props.checked, ""])
    }
    else
    {
        icon_color.value = '#d77a37'
        information.value = "Необходимо осуществить проверку документа";
    }
})

times.value.sort((a ,b) => a[2].date.greater_or_equal_then(b[2].date.self()) ? 0 : 1)
</script>
    
<style lang="scss" scoped>
.statuses
{
    display: flex;
    flex-direction: column;
    gap: 3px;
    
}
.status
{
    display: flex;
    align-content: baseline;
    gap: 3px;
    background: rgba(75, 145, 226, 0.12);
    box-shadow: 0 8px 32px 0 rgba( 31, 38, 135, 0.37 );
    backdrop-filter: blur( 8px );
    -webkit-backdrop-filter: blur( 8px );
    border-radius: 32px;
    margin-top: 2px;
    border: 1px solid rgba( 255, 255, 255, 0.18 );
}
.text
{
    font-size: 16px;
    font-weight: 600;
    margin-right: 3px;
}
.status-icon
{
    display: flex;
    align-items: center;
    gap: 3px;
   
}
.text-field
{
    display: flex;
    flex-direction: column;
    gap: 1px;
   
}
.user-icon
{
    justify-self: center;
    align-self: center;
}
</style>