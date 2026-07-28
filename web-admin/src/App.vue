<template>
  <n-config-provider :theme="theme">
    <n-message-provider>
      <n-dialog-provider>
        <n-layout has-sider style="height: 100vh">
          <n-layout-sider
            bordered
            :collapsed="collapsed"
            :collapsed-width="64"
            :width="240"
            show-trigger
            @collapse="collapsed = true"
            @expand="collapsed = false"
          >
            <div class="logo">
              <span v-if="!collapsed">AI 技能百宝箱</span>
              <span v-else>AI</span>
            </div>
            <n-menu
              :collapsed="collapsed"
              :collapsed-width="64"
              :collapsed-icon-size="22"
              :options="menuOptions"
              :value="activeMenu"
              @update:value="handleMenuUpdate"
            />
          </n-layout-sider>
          <n-layout-content content-style="padding: 24px;" :native-scrollbar="false">
            <router-view v-slot="{ Component }">
              <component :is="Component" :key="route.fullPath" />
            </router-view>
          </n-layout-content>
        </n-layout>
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>

<script setup>
import { ref, computed, h } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { NIcon } from 'naive-ui'
import {
  GridOutline,
  ListOutline,
  TrashOutline,
  SettingsOutline
} from '@vicons/ionicons5'

const router = useRouter()
const route = useRoute()
const collapsed = ref(false)
const theme = ref(null)

const activeMenu = computed(() => {
  const path = route.path
  if (path === '/') return '/'
  if (path.startsWith('/skills')) return '/skills'
  if (path.startsWith('/trash')) return '/trash'
  if (path.startsWith('/settings')) return '/settings'
  return path
})

const menuOptions = [
  {
    label: '仪表盘',
    key: '/',
    icon: () => h(NIcon, null, { default: () => h(GridOutline) })
  },
  {
    label: 'Skill管理',
    key: '/skills',
    icon: () => h(NIcon, null, { default: () => h(ListOutline) })
  },
  {
    label: '回收站',
    key: '/trash',
    icon: () => h(NIcon, null, { default: () => h(TrashOutline) })
  },
  {
    label: '设置',
    key: '/settings',
    icon: () => h(NIcon, null, { default: () => h(SettingsOutline) })
  }
]

const handleMenuUpdate = (key) => {
  if (route.path !== key) {
    router.push(key).catch(() => {})
  }
}
</script>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
}

.logo {
  height: 64px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  font-weight: 600;
  border-bottom: 1px solid #e0e0e0;
}
</style>
