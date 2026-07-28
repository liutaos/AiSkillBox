// Copyright (c) Mr_老鬼. All rights reserved.
// https://www.junjiestudio.top
import { createApp } from 'vue'
import naive from 'naive-ui'
import App from './App.vue'
import router from './router'

const app = createApp(App)
app.use(naive)
app.use(router)
app.mount('#app')
