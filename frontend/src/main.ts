import { createApp } from "vue";
import 'vue-virtual-scroller/dist/vue-virtual-scroller.css'
import App from "./App.vue";
import emitter from './services/emitter'
import sse_service from '@/services/sse'
import router from "./router";
import plugin from 'vue-toastify';
// base styles
import 'vue-toastify/index.css';
// theme styles
import 'vue-toastify/themes/dark.css';
import type { Settings } from 'vue-toastify';
import { title } from "process";
import '@styles/index.scss'

const app = createApp(App);
app.provide('emitter', emitter);
app.provide('sse', sse_service);
app.use(router)
app.use<Settings>(plugin, 
{
    position: 'top-right',
    customNotifications: 
    {
        authenticationError:
        {
            
            type: undefined
            // ... rest of the toast options here
        }
    }
});
app.mount("#app");
