<template lang="pug">
.view-wrapper
    documents-list
    pdf-viewer-drawler
</template>
    
<script lang="ts">
import { ref, type Component, watch, inject, onMounted, onUnmounted, computed, onBeforeUnmount, h, toRefs } from 'vue';
import { type Events, type Emitter } from '../services/emitter';
import { NForm, NFormItem, NInput, NButton, darkTheme } from 'naive-ui';
import { notify_service } from '@/services/notification_service';
import {DocumentsList} from '@components/documents/document_list'
import PdfViewerDrawler from '@/components/PdfViewerDrawler.vue';
//import  user_service  from '../services/user_service';
</script>
<script lang="ts" setup>
const emitter = inject<Emitter<Events>>('emitter') as Emitter<Events>;
</script>
    
<style lang="scss" scoped>

</style>