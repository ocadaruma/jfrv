(function(){"use strict";var e={704:function(e,t,n){var o=n(9242),a=n(3396);const r=e=>((0,a.dD)("data-v-6fd9328d"),e=e(),(0,a.Cn)(),e),l={class:"h-12 fixed top-0 left-0 z-50 w-full bg-white border-b border-slate-400"},i={class:"h-8 m-2 flex space-x-2"},u=r((()=>(0,a._)("span",{class:"align-middle"},"execution sample",-1))),s=r((()=>(0,a._)("span",{class:"align-middle"},"offcpu (jvm-blocking-monitor)",-1))),c=r((()=>(0,a._)("a",{href:"https://github.com/ocadaruma/jfrv",target:"_blank",class:"absolute h-7 border-2 border-slate-500 pl-2 pr-2 right-2"},"📖 docs",-1))),d={class:"fixed top-12 left-0 right-0 bottom-0"};function f(e,t){const n=(0,a.up)("router-link"),o=(0,a.up)("router-view");return(0,a.wg)(),(0,a.iD)("div",null,[(0,a._)("div",l,[(0,a._)("nav",i,[(0,a.Wm)(n,{to:"/execution-sample",class:"hover:bg-slate-300 flex-none pl-2 pr-2 h-7 text-sm text-center border-2 rounded border-slate-400"},{default:(0,a.w5)((()=>[u])),_:1}),(0,a.Wm)(n,{to:"/jbm",class:"hover:bg-slate-300 flex-none pl-2 pr-2 h-7 text-sm text-center border-2 rounded border-slate-400"},{default:(0,a.w5)((()=>[s])),_:1}),c])]),(0,a._)("div",d,[(0,a.Wm)(o,null,{default:(0,a.w5)((({Component:e})=>[((0,a.wg)(),(0,a.j4)(a.Ob,null,[((0,a.wg)(),(0,a.j4)((0,a.LL)(e)))],1024))])),_:1})])])}var v=n(89);const h={},p=(0,v.Z)(h,[["render",f],["__scopeId","data-v-6fd9328d"]]);var m=p,g=n(2483),b=(n(8675),n(3462),n(7380),n(1118),n(4870)),w=n(7139),_=n(3581),y=n(8430);const x={class:"fixed top-12 left-0 right-0 h-12 bg-neutral-100 z-40 border-b border-slate-400 p-2"},k=(0,a._)("span",{class:"h-7 ml-2"},"thread name:",-1),S=(0,a._)("div",{class:"flex flex-col space-x-2"},null,-1),j={class:"absolute top-12 right-0 left-0 bottom-0"},H={class:"w-full h-full"},C={ref:"header-overlay",id:"header-overlay",class:"absolute top-0 left-0 pointer-events-none",width:"0",height:"0"},R={ref:"chart-overlay",id:"chart-overlay",class:"absolute top-0 left-0 pointer-events-none",width:"0",height:"0"},U={key:0},D={key:0},W={key:1},z={class:"p-2"},O={key:0,class:"fixed w-72 h-24 bg-neutral-200 border-neutral-500 p-2 border-2 top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2"};var E=(0,a.aZ)({__name:"ExecutionSample",setup(e){const t={defaultMargin:1,fontSize:14,headerConfig:{borderWidth:1,borderColorRgbHex:7368816,elementId:"header",overlayElementId:"header-overlay"},sampleViewConfig:{elementId:"thread-chart-sample-view",overlayElementId:"chart-overlay",sampleRenderSize:{width:6,height:8},backgroundRgbHex:16119285},threadStateColorConfig:{stateRunnableRgbHex:7125534,stateSleepingRgbHex:8737986,stateUnknownRgbHex:7302514},overlayConfig:{rowHighlightArgbHex:1077952576,sampleHighlightRgbHex:15745140}},r=(0,b.iH)(),l=(0,b.iH)(),i=(0,b.iH)(),u=(0,b.iH)(),s=(0,b.iH)(),c=(0,b.iH)(),d=(0,b.iH)(),f=(0,b.iH)(),{getRootProps:v,getInputProps:h,isDragActive:p,open:m}=(0,y.u)({onDrop:A,multiple:!1,noClick:!0,noKeyboard:!0,accept:".jfr"});function g(){f.value&&(void 0!==d.value&&d.value?.length>0?r.value?.apply_filter({threadNameRegex:d.value}):r.value?.apply_filter({threadNameRegex:null}))}function E(){const e=r.value?.on_chart_click();l.value=e?.frames}function P(e){r.value?.on_header_mouse_move(e.clientX-s.value.getBoundingClientRect().x,e.clientY-s.value.getBoundingClientRect().y)}function M(e){r.value?.on_chart_mouse_move(e.clientX-c.value.getBoundingClientRect().x,e.clientY-c.value.getBoundingClientRect().y)}function T(){r.value?.on_mouse_out()}async function A(e,t){f.value="loading";const n=await e[0].arrayBuffer(),o=new Uint8Array(n);await X(o)}async function B(){f.value="loading";const e=await fetch("/jfrv/demo.jfr"),t=await e.arrayBuffer(),n=new Uint8Array(t);await X(n)}async function X(e){d.value=void 0;try{r.value?.initialize(e),r.value?.render()}catch(t){throw f.value=void 0,t}f.value="loaded"}(0,a.bv)((async()=>{const e=await n.e(672).then(n.bind(n,7672));r.value=new e.Renderer(t)}));const I=e=>{const t=i.value?.$el,n=u.value?.$el;"header"===e&&(n.scrollTop=t.scrollTop),"chart"===e&&(t.scrollTop=n.scrollTop)};return(e,t)=>((0,a.wg)(),(0,a.iD)(a.HY,null,[(0,a._)("div",x,[(0,a._)("button",{class:"hover:bg-slate-300 w-24 h-7 text-sm text-center border-2 rounded border-slate-400",onClick:t[0]||(t[0]=(...e)=>(0,b.SU)(m)&&(0,b.SU)(m)(...e))},"open file"),(0,a._)("button",{class:"hover:bg-slate-300 w-24 h-7 text-sm text-center border-2 border-slate-500 absolute right-2",onClick:B},"load demo"),k,(0,a.wy)((0,a._)("input",{class:"h-7",type:"text",placeholder:"regex","onUpdate:modelValue":t[1]||(t[1]=e=>d.value=e),onChange:g},null,544),[[o.nr,d.value]]),(0,a._)("input",(0,w.vs)((0,a.F4)((0,b.SU)(h)())),null,16),S]),(0,a._)("div",j,[(0,a.Wm)((0,b.SU)(_.F),(0,a.dG)({class:"default-theme text-sm",horizontal:""},(0,b.SU)(v)()),{default:(0,a.w5)((()=>[(0,a.Wm)((0,b.SU)(_.X),null,{default:(0,a.w5)((()=>[(0,a._)("div",H,[(0,a.Wm)((0,b.SU)(_.F),{vertical:""},{default:(0,a.w5)((()=>[(0,a.Wm)((0,b.SU)(_.X),{size:"25",class:"overflow-x-hidden overflow-y-auto scrollbar-none relative",onScroll:t[2]||(t[2]=e=>I("header")),ref_key:"headerPane",ref:i},{default:(0,a.w5)((()=>[(0,a._)("canvas",C,null,512),((0,a.wg)(),(0,a.iD)("svg",{ref_key:"header",ref:s,id:"header",class:"absolute top-0 left-0",onMousemove:P,onMouseout:T,width:"0",height:"0"},null,544))])),_:1},512),(0,a.Wm)((0,b.SU)(_.X),{class:"overflow-auto relative",onScroll:t[3]||(t[3]=e=>I("chart")),ref_key:"chartPane",ref:u},{default:(0,a.w5)((()=>[(0,a._)("canvas",R,null,512),(0,a._)("canvas",{ref_key:"chart",ref:c,id:"thread-chart-sample-view",onMousemove:M,onMouseout:T,onClick:E,class:"bg-slate-100",width:"0",height:"0"},null,544),f.value?(0,a.kq)("",!0):((0,a.wg)(),(0,a.iD)("div",U,[(0,b.SU)(p)?((0,a.wg)(),(0,a.iD)("p",D,"Drop here ...")):((0,a.wg)(),(0,a.iD)("p",W,'Drag & drop OR press "open file" to select JFR file'))]))])),_:1},512)])),_:1})])])),_:1}),(0,a.Wm)((0,b.SU)(_.X),{size:"40",class:"overflow-auto"},{default:(0,a.w5)((()=>[(0,a._)("div",z,[((0,a.wg)(!0),(0,a.iD)(a.HY,null,(0,a.Ko)(l.value,((e,t)=>((0,a.wg)(),(0,a.iD)("div",{class:"flex flex-col space-x-2 text-sm",key:t},(0,w.zw)(e.typeName)+"@"+(0,w.zw)(e.methodName),1)))),128))])])),_:1})])),_:1},16)]),"loading"===f.value?((0,a.wg)(),(0,a.iD)("div",O,"Loading...")):(0,a.kq)("",!0)],64))}});const P=E;var M=P;const T={class:"fixed top-12 left-0 right-0 h-12 bg-neutral-100 z-40 border-b border-slate-400 p-2"},A=(0,a._)("span",{class:"h-7 ml-2"},"thread name:",-1),B=(0,a._)("div",{class:"flex flex-col space-x-2"},null,-1),X={class:"absolute top-12 right-0 left-0 bottom-0"},I={class:"w-full h-full"},N={ref:"header-overlay",id:"jbm-header-overlay",class:"absolute top-0 left-0 pointer-events-none",width:"0",height:"0"},F={ref:"chart-overlay",id:"jbm-chart-overlay",class:"absolute top-0 left-0 pointer-events-none",width:"0",height:"0"},Y={key:0},q={key:0},L={key:1},K={class:"w-full h-full"},V={class:"p-2"},$={class:"h-full p-2 overflow-auto"},J={class:"table-auto whitespace-nowrap"},Z=(0,a._)("td",{class:"text-right"},"thread :",-1),G=(0,a._)("td",{class:"text-right"},"offcpu start :",-1),Q=(0,a._)("td",{class:"text-right"},"offcpu end :",-1),ee=(0,a._)("td",{class:"text-right"},"duration (ms) :",-1),te={key:0,class:"fixed w-72 h-24 bg-neutral-200 border-neutral-500 p-2 border-2 top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2"};var ne=(0,a.aZ)({__name:"JvmBlockingMonitor",setup(e){const t={defaultMargin:1,fontSize:14,headerConfig:{borderWidth:1,borderColorRgbHex:7368816,elementId:"jbm-header",overlayElementId:"jbm-header-overlay"},sampleViewConfig:{elementId:"jbm-thread-chart-sample-view",overlayElementId:"jbm-chart-overlay",sampleRenderHeight:8,sampleWidthPerHour:256,backgroundRgbHex:16119285},threadStateColorConfig:{stateRunnableRgbHex:7125534,stateSleepingRgbHex:8737986,stateUnknownRgbHex:7302514},overlayConfig:{rowHighlightArgbHex:1077952576,sampleHighlightRgbHex:15745140}},r=(0,b.iH)(),l=(0,b.iH)(),i=(0,b.iH)(),u=(0,b.iH)(),s=(0,b.iH)(),c=(0,b.iH)(),d=(0,b.iH)(),f=(0,b.iH)(),{getRootProps:v,getInputProps:h,isDragActive:p,open:m}=(0,y.u)({onDrop:H,multiple:!1,noClick:!0,noKeyboard:!0});function g(){f.value&&(void 0!==d.value&&d.value?.length>0?r.value?.apply_filter({threadNameRegex:d.value}):r.value?.apply_filter({threadNameRegex:null}))}function x(){l.value=r.value?.on_chart_click()}function k(e){r.value?.on_header_mouse_move(e.clientX-s.value.getBoundingClientRect().x,e.clientY-s.value.getBoundingClientRect().y)}function S(e){r.value?.on_chart_mouse_move(e.clientX-c.value.getBoundingClientRect().x,e.clientY-c.value.getBoundingClientRect().y)}function j(){r.value?.on_mouse_out()}async function H(e,t){f.value="loading";const n=await e[0].arrayBuffer(),o=new Uint8Array(n);await R(o)}async function C(){f.value="loading";const e=await fetch("/jfrv/jbm.log"),t=await e.arrayBuffer(),n=new Uint8Array(t);await R(n)}async function R(e){d.value=void 0;try{r.value?.initialize(e),r.value?.render()}catch(t){throw f.value=void 0,t}f.value="loaded"}(0,a.bv)((async()=>{const e=await n.e(672).then(n.bind(n,7672));r.value=new e.JbmRenderer(t)}));const U=e=>{const t=i.value?.$el,n=u.value?.$el;"header"===e&&(n.scrollTop=t.scrollTop),"chart"===e&&(t.scrollTop=n.scrollTop)};return(e,t)=>((0,a.wg)(),(0,a.iD)(a.HY,null,[(0,a._)("div",T,[(0,a._)("button",{class:"hover:bg-slate-300 w-24 h-7 text-sm text-center border-2 rounded border-slate-400",onClick:t[0]||(t[0]=(...e)=>(0,b.SU)(m)&&(0,b.SU)(m)(...e))},"open file"),(0,a._)("button",{class:"hover:bg-slate-300 w-24 h-7 text-sm text-center border-2 border-slate-500 absolute right-2",onClick:C},"load demo"),A,(0,a.wy)((0,a._)("input",{class:"h-7",type:"text",placeholder:"regex","onUpdate:modelValue":t[1]||(t[1]=e=>d.value=e),onChange:g},null,544),[[o.nr,d.value]]),(0,a._)("input",(0,w.vs)((0,a.F4)((0,b.SU)(h)())),null,16),B]),(0,a._)("div",X,[(0,a.Wm)((0,b.SU)(_.F),(0,a.dG)({class:"default-theme text-sm",horizontal:""},(0,b.SU)(v)()),{default:(0,a.w5)((()=>[(0,a.Wm)((0,b.SU)(_.X),null,{default:(0,a.w5)((()=>[(0,a._)("div",I,[(0,a.Wm)((0,b.SU)(_.F),{vertical:""},{default:(0,a.w5)((()=>[(0,a.Wm)((0,b.SU)(_.X),{size:"25",class:"overflow-x-hidden overflow-y-auto scrollbar-none relative",onScroll:t[2]||(t[2]=e=>U("header")),ref_key:"headerPane",ref:i},{default:(0,a.w5)((()=>[(0,a._)("canvas",N,null,512),((0,a.wg)(),(0,a.iD)("svg",{ref_key:"header",ref:s,id:"jbm-header",class:"absolute top-0 left-0",onMousemove:k,onMouseout:j,width:"0",height:"0"},null,544))])),_:1},512),(0,a.Wm)((0,b.SU)(_.X),{class:"overflow-auto relative",onScroll:t[3]||(t[3]=e=>U("chart")),ref_key:"chartPane",ref:u},{default:(0,a.w5)((()=>[(0,a._)("canvas",F,null,512),(0,a._)("canvas",{ref_key:"chart",ref:c,id:"jbm-thread-chart-sample-view",onMousemove:S,onMouseout:j,onClick:x,class:"bg-slate-100",width:"0",height:"0"},null,544),f.value?(0,a.kq)("",!0):((0,a.wg)(),(0,a.iD)("div",Y,[(0,b.SU)(p)?((0,a.wg)(),(0,a.iD)("p",q,"Drop here ...")):((0,a.wg)(),(0,a.iD)("p",L,'Drag & drop OR press "open file" to select jbm log file'))]))])),_:1},512)])),_:1})])])),_:1}),(0,a.Wm)((0,b.SU)(_.X),{size:"40"},{default:(0,a.w5)((()=>[(0,a._)("div",K,[(0,a.Wm)((0,b.SU)(_.F),{vertical:""},{default:(0,a.w5)((()=>[(0,a.Wm)((0,b.SU)(_.X),{size:"75",class:"overflow-auto"},{default:(0,a.w5)((()=>[(0,a._)("div",V,[((0,a.wg)(!0),(0,a.iD)(a.HY,null,(0,a.Ko)(l.value?.stackTrace?.frames,((e,t)=>((0,a.wg)(),(0,a.iD)("div",{class:"flex flex-col space-x-2 text-sm",key:t},(0,w.zw)(e.methodName),1)))),128))])])),_:1}),(0,a.Wm)((0,b.SU)(_.X),null,{default:(0,a.w5)((()=>[(0,a._)("div",$,[(0,a._)("table",J,[(0,a._)("tbody",null,[(0,a._)("tr",null,[Z,(0,a._)("td",null,(0,w.zw)(l.value?.threadName),1)]),(0,a._)("tr",null,[G,(0,a._)("td",null,(0,w.zw)(l.value?.offcpuStart),1)]),(0,a._)("tr",null,[Q,(0,a._)("td",null,(0,w.zw)(l.value?.offcpuEnd),1)]),(0,a._)("tr",null,[ee,(0,a._)("td",null,(0,w.zw)(l.value?.durationMillis),1)])])])])])),_:1})])),_:1})])])),_:1})])),_:1},16)]),"loading"===f.value?((0,a.wg)(),(0,a.iD)("div",te,"Loading...")):(0,a.kq)("",!0)],64))}});const oe=ne;var ae=oe;const re=[{path:"/",redirect:"/execution-sample"},{path:"/execution-sample",name:"execution-sample",component:M},{path:"/jbm",name:"jbm",component:ae}],le=(0,g.p7)({history:(0,g.r5)("/jfrv/"),routes:re});var ie=le;(0,o.ri)(m).use(ie).mount("#app")}},t={};function n(o){var a=t[o];if(void 0!==a)return a.exports;var r=t[o]={id:o,loaded:!1,exports:{}};return e[o](r,r.exports,n),r.loaded=!0,r.exports}n.m=e,function(){var e="function"===typeof Symbol?Symbol("webpack queues"):"__webpack_queues__",t="function"===typeof Symbol?Symbol("webpack exports"):"__webpack_exports__",o="function"===typeof Symbol?Symbol("webpack error"):"__webpack_error__",a=function(e){e&&!e.d&&(e.d=1,e.forEach((function(e){e.r--})),e.forEach((function(e){e.r--?e.r++:e()})))},r=function(n){return n.map((function(n){if(null!==n&&"object"===typeof n){if(n[e])return n;if(n.then){var r=[];r.d=0,n.then((function(e){l[t]=e,a(r)}),(function(e){l[o]=e,a(r)}));var l={};return l[e]=function(e){e(r)},l}}var i={};return i[e]=function(){},i[t]=n,i}))};n.a=function(n,l,i){var u;i&&((u=[]).d=1),u&&(u.moduleId=n.id);var s,c,d,f=new Set,v=n.exports,h=new Promise((function(e,t){d=t,c=e}));h[t]=v,h[e]=function(e){u&&e(u),f.forEach(e),h["catch"]((function(){}))},h.moduleId=n.id,n.exports=h,l((function(n){var a;s=r(n);var l=function(){return s.map((function(e){if(e[o])throw e[o];return e[t]}))},i=new Promise((function(t){a=function(){t(l)},a.r=0;var n=function(e){e!==u&&!f.has(e)&&(f.add(e),e&&!e.d&&(a.r++,e.push(a)))};s.map((function(t){t[e](n)}))}));return a.r?i:l()}),(function(e){e?d(h[o]=e):c(v),a(u)})),u&&(u.d=0)}}(),function(){var e=[];n.O=function(t,o,a,r){if(!o){var l=1/0;for(c=0;c<e.length;c++){o=e[c][0],a=e[c][1],r=e[c][2];for(var i=!0,u=0;u<o.length;u++)(!1&r||l>=r)&&Object.keys(n.O).every((function(e){return n.O[e](o[u])}))?o.splice(u--,1):(i=!1,r<l&&(l=r));if(i){e.splice(c--,1);var s=a();void 0!==s&&(t=s)}}return t}r=r||0;for(var c=e.length;c>0&&e[c-1][2]>r;c--)e[c]=e[c-1];e[c]=[o,a,r]}}(),function(){n.n=function(e){var t=e&&e.__esModule?function(){return e["default"]}:function(){return e};return n.d(t,{a:t}),t}}(),function(){n.d=function(e,t){for(var o in t)n.o(t,o)&&!n.o(e,o)&&Object.defineProperty(e,o,{enumerable:!0,get:t[o]})}}(),function(){n.f={},n.e=function(e){return Promise.all(Object.keys(n.f).reduce((function(t,o){return n.f[o](e,t),t}),[]))}}(),function(){n.u=function(e){return"js/"+e+".3665278b.js"}}(),function(){n.miniCssF=function(e){}}(),function(){n.g=function(){if("object"===typeof globalThis)return globalThis;try{return this||new Function("return this")()}catch(e){if("object"===typeof window)return window}}()}(),function(){n.hmd=function(e){return e=Object.create(e),e.children||(e.children=[]),Object.defineProperty(e,"exports",{enumerable:!0,set:function(){throw new Error("ES Modules may not assign module.exports or exports.*, Use ESM export syntax, instead: "+e.id)}}),e}}(),function(){n.o=function(e,t){return Object.prototype.hasOwnProperty.call(e,t)}}(),function(){var e={},t="jfrv:";n.l=function(o,a,r,l){if(e[o])e[o].push(a);else{var i,u;if(void 0!==r)for(var s=document.getElementsByTagName("script"),c=0;c<s.length;c++){var d=s[c];if(d.getAttribute("src")==o||d.getAttribute("data-webpack")==t+r){i=d;break}}i||(u=!0,i=document.createElement("script"),i.charset="utf-8",i.timeout=120,n.nc&&i.setAttribute("nonce",n.nc),i.setAttribute("data-webpack",t+r),i.src=o),e[o]=[a];var f=function(t,n){i.onerror=i.onload=null,clearTimeout(v);var a=e[o];if(delete e[o],i.parentNode&&i.parentNode.removeChild(i),a&&a.forEach((function(e){return e(n)})),t)return t(n)},v=setTimeout(f.bind(null,void 0,{type:"timeout",target:i}),12e4);i.onerror=f.bind(null,i.onerror),i.onload=f.bind(null,i.onload),u&&document.head.appendChild(i)}}}(),function(){n.r=function(e){"undefined"!==typeof Symbol&&Symbol.toStringTag&&Object.defineProperty(e,Symbol.toStringTag,{value:"Module"}),Object.defineProperty(e,"__esModule",{value:!0})}}(),function(){n.v=function(e,t,o,a){var r=fetch(n.p+""+o+".module.wasm");return"function"===typeof WebAssembly.instantiateStreaming?WebAssembly.instantiateStreaming(r,a).then((function(t){return Object.assign(e,t.instance.exports)})):r.then((function(e){return e.arrayBuffer()})).then((function(e){return WebAssembly.instantiate(e,a)})).then((function(t){return Object.assign(e,t.instance.exports)}))}}(),function(){n.p="/jfrv/"}(),function(){var e={143:0};n.f.j=function(t,o){var a=n.o(e,t)?e[t]:void 0;if(0!==a)if(a)o.push(a[2]);else{var r=new Promise((function(n,o){a=e[t]=[n,o]}));o.push(a[2]=r);var l=n.p+n.u(t),i=new Error,u=function(o){if(n.o(e,t)&&(a=e[t],0!==a&&(e[t]=void 0),a)){var r=o&&("load"===o.type?"missing":o.type),l=o&&o.target&&o.target.src;i.message="Loading chunk "+t+" failed.\n("+r+": "+l+")",i.name="ChunkLoadError",i.type=r,i.request=l,a[1](i)}};n.l(l,u,"chunk-"+t,t)}},n.O.j=function(t){return 0===e[t]};var t=function(t,o){var a,r,l=o[0],i=o[1],u=o[2],s=0;if(l.some((function(t){return 0!==e[t]}))){for(a in i)n.o(i,a)&&(n.m[a]=i[a]);if(u)var c=u(n)}for(t&&t(o);s<l.length;s++)r=l[s],n.o(e,r)&&e[r]&&e[r][0](),e[r]=0;return n.O(c)},o=self["webpackChunkjfrv"]=self["webpackChunkjfrv"]||[];o.forEach(t.bind(null,0)),o.push=t.bind(null,o.push.bind(o))}();var o=n.O(void 0,[998],(function(){return n(704)}));o=n.O(o)})();
//# sourceMappingURL=app.93589649.js.map