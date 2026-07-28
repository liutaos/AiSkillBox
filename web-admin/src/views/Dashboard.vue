<template>
  <div class="dashboard">
    <h1>仪表盘</h1>
    
    <!-- 统计卡片 -->
    <n-grid :cols="4" :x-gap="16">
      <n-gi>
        <n-card class="stat-card">
          <n-statistic label="Skill总数" :value="stats.total" />
        </n-card>
      </n-gi>
      <n-gi>
        <n-card class="stat-card">
          <n-statistic label="已启用" :value="stats.enabled" />
        </n-card>
      </n-gi>
      <n-gi>
        <n-card class="stat-card">
          <n-statistic label="已禁用" :value="stats.disabled" />
        </n-card>
      </n-gi>
      <n-gi>
        <n-card class="stat-card">
          <n-statistic label="回收站" :value="stats.trash" />
        </n-card>
      </n-gi>
    </n-grid>
    
    <!-- 服务状态 -->
    <n-card title="服务状态" style="margin-top: 24px">
      <n-space>
        <n-tag :type="status.running ? 'success' : 'error'" size="large">
          {{ status.running ? '运行中' : '已停止' }}
        </n-tag>
        <n-text>MCP端口: {{ config.mcp_listen_addr || '-' }}</n-text>
        <n-text>Web端口: {{ config.listen_addr || '-' }}</n-text>
      </n-space>
      
      <n-space style="margin-top: 16px">
        <n-button 
          v-if="!status.running" 
          type="primary" 
          @click="handleStart"
          :loading="loading.start"
        >
          启动服务
        </n-button>
        <n-button 
          v-else 
          type="error" 
          @click="handleStop"
          :loading="loading.stop"
        >
          停止服务
        </n-button>
        <n-button 
          @click="handleRestart"
          :loading="loading.restart"
          :disabled="!status.running"
        >
          重启服务
        </n-button>
        <n-button 
          @click="handleRefresh"
          :loading="loading.refresh"
        >
          刷新列表
        </n-button>
      </n-space>
    </n-card>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { useMessage } from 'naive-ui'
import { 
  getSkills, 
  getTrash, 
  getStatus, 
  getConfig,
  startService,
  stopService,
  restartService,
  refreshSkills
} from '../api'

const message = useMessage()

const stats = ref({
  total: 0,
  enabled: 0,
  disabled: 0,
  trash: 0
})

const status = ref({
  running: false
})

const config = ref({})

const loading = ref({
  start: false,
  stop: false,
  restart: false,
  refresh: false
})

const fetchData = async () => {
  try {
    const [skillsRes, trashRes, statusRes, configRes] = await Promise.all([
      getSkills(),
      getTrash(),
      getStatus(),
      getConfig()
    ])
    
    stats.value.total = skillsRes.data?.count || 0
    stats.value.enabled = skillsRes.data?.skills?.filter(s => s.enabled).length || 0
    stats.value.disabled = skillsRes.data?.skills?.filter(s => !s.enabled).length || 0
    stats.value.trash = trashRes.data?.count || 0
    
    status.value = statusRes.data || {}
    config.value = configRes.data || {}
  } catch (error) {
    message.error('加载数据失败: ' + error.message)
  }
}

const handleStart = async () => {
  loading.value.start = true
  try {
    await startService()
    message.success('服务启动成功')
    await fetchData()
  } catch (error) {
    message.error('启动失败: ' + error.message)
  } finally {
    loading.value.start = false
  }
}

const handleStop = async () => {
  loading.value.stop = true
  try {
    await stopService()
    message.success('服务已停止')
    await fetchData()
  } catch (error) {
    message.error('停止失败: ' + error.message)
  } finally {
    loading.value.stop = false
  }
}

const handleRestart = async () => {
  loading.value.restart = true
  try {
    await restartService()
    message.success('服务重启成功')
    await fetchData()
  } catch (error) {
    message.error('重启失败: ' + error.message)
  } finally {
    loading.value.restart = false
  }
}

const handleRefresh = async () => {
  loading.value.refresh = true
  try {
    await refreshSkills()
    message.success('刷新成功')
    await fetchData()
  } catch (error) {
    message.error('刷新失败: ' + error.message)
  } finally {
    loading.value.refresh = false
  }
}

onMounted(fetchData)
</script>

<style scoped>
.dashboard h1 {
  margin-bottom: 24px;
  font-size: 28px;
  font-weight: 600;
}

.stat-card {
  text-align: center;
}
</style>
