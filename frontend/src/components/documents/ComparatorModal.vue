<template lang="pug">
.panel
    dynamic-render(:doc_id="props.doc_id")
    pdf-viewer(:doc_id="props.doc_id")
</template>
    
<script lang="ts">
import { ref, type Component, watch, inject, onMounted, onUnmounted, computed, onBeforeUnmount, h, toRefs } from 'vue';
import DynamicRender from '../DynamicRender.vue';
import PdfViewer from '../PdfViewer.vue';
</script>
<script lang="ts" setup>

const props = defineProps<{
    doc_id: number
}>();


</script>
    
<style lang="scss" scoped>
.panel
{
    display: flex;
    flex-direction: row;
    max-height: 90vh;
}
</style>