import { createApp } from 'vue'
import App from './App.vue'
import router from './router'
import './main.css'
import '@finos/perspective-viewer';
import '@finos/perspective-viewer-datagrid';
import '@finos/perspective-viewer-d3fc';
import "@finos/perspective-viewer/dist/css/solarized.css";
import VueCodemirror from 'vue-codemirror';
import {basicSetup} from "codemirror";

createApp(App)
    .use(router)
    .use(VueCodemirror, {
        extensions: [basicSetup]
    })
    .mount('#app')
