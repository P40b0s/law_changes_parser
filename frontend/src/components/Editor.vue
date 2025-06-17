<template lang="pug">
div
	div(ref="editorRef" id="editorjs")
	button(@click="handleSave" :disabled="isSaving") {{ isSaving ? 'Saving...' : 'Save' }}
	div(v-if="error" class="error-message") {{ error }}
</template>
  
<script lang="ts">
import { defineComponent, ref, onMounted, onBeforeUnmount } from 'vue';
import EditorJS, { type OutputData } from '@editorjs/editorjs';
import Paragraph from '@editorjs/paragraph';
import Table from '@editorjs/table';
import { http_sevice } from '@/services/http_service';

</script>
<script lang="ts" setup>
interface Props 
{
    doc_id: number,
}
const props = defineProps<Props>();
const editorRef = ref<HTMLElement | null>(null);
const editorInstance = ref<EditorJS | null>(null);
const editor_data = ref<OutputData | undefined>();
const isSaving = ref(false);
const error = ref<string | null>(null);

onMounted(async () =>
{
    const data = await http_sevice.get_document_for_editorjs(props.doc_id);
    editor_data.value = data;

    editorInstance.value = new EditorJS({
        holder: 'editorjs',
        inlineToolbar:['bold', 'italic', 'underline'] ,
        tools: 
        {
            paragraph:  Paragraph,
            
            table: Table
        },
        data: editor_data.value,
        onReady: () => 
        {
            console.log('Editor.js is ready');
        },
        onChange: () => 
        {
            error.value = null; // Clear errors on change
        },
    });
})

// const initializeEditor = async () => 
// {
//     try {
//         // Fetch data from backend
//         const response: AxiosResponse<EditorJsResponse> = await axios.get(
//         `${props.apiBaseUrl}/api/document/${props.documentId}`
//         );
        
//         editorData.value = {
//         time: response.data.time,
//         blocks: response.data.blocks,
//         version: response.data.version,
//         };

//         // Initialize Editor.js
       
//     } catch (err) {
//         handleError(err, 'Failed to initialize editor');
//     }
//     };

    const handleSave = async () => 
    {
        if (!editorInstance.value) return;
        console.log("TRY SAVE!")
        isSaving.value = true;
        error.value = null;

        // try {
        //     const outputData = await editorInstance.value.save();
            
        //     const response: AxiosResponse<{ status: string }> = await axios.post(
        //     `${props.apiBaseUrl}/api/document/${props.documentId}`,
        //     outputData
        //     );

        //     if (response.data.status === 'ok') {
        //     console.log('Document saved successfully');
        //     } else {
        //     throw new Error('Unexpected response from server');
        //     }
        // } catch (err) {
        //     handleError(err, 'Failed to save document');
        // } finally {
        //     isSaving.value = false;
        // }
    };

    onBeforeUnmount(() => {
    if (editorInstance.value) {
        editorInstance.value.destroy();
    }
    });

</script>
  
<style scoped lang="scss">
#editorjs {
border: 1px solid #ddd;
padding: 10px;
margin-bottom: 10px;
min-height: 200px;
}

button {
padding: 8px 16px;
background-color: #42b983;
color: white;
border: none;
border-radius: 4px;
cursor: pointer;
}

button:disabled {
background-color: #cccccc;
cursor: not-allowed;
}

.error-message {
color: #ff4444;
margin-top: 10px;
}
</style>