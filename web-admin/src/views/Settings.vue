<template>
  <div class="settings">
    <h1>设置</h1>
    
    <!-- 服务配置 -->
    <n-card title="服务配置">
      <n-descriptions :column="1" bordered>
        <n-descriptions-item label="MCP端口">
          {{ config.mcp_listen_addr || '-' }}
        </n-descriptions-item>
        <n-descriptions-item label="Web端口">
          {{ config.listen_addr || '-' }}
        </n-descriptions-item>
        <n-descriptions-item label="数据库路径">
          {{ config.db_path || '-' }}
        </n-descriptions-item>
        <n-descriptions-item label="日志级别">
          {{ config.log_level || '-' }}
        </n-descriptions-item>
      </n-descriptions>
    </n-card>
    
    <!-- 服务控制 -->
    <n-card title="服务控制" style="margin-top: 16px">
      <n-space>
        <n-button 
          type="primary" 
          @click="handleStart"
          :loading="loading.start"
        >
          启动服务
        </n-button>
        <n-button 
          type="error" 
          @click="handleStop"
          :loading="loading.stop"
        >
          停止服务
        </n-button>
        <n-button 
          @click="handleRestart"
          :loading="loading.restart"
        >
          重启服务
        </n-button>
      </n-space>
    </n-card>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { useMessage } from 'naive-ui'
import { getConfig, startService, stopService, restartService } from '../api'

const message = useMessage()

const config = ref({})

const loading = ref({
  start: false,
  stop: false,
  restart: false
})

const fetchConfig = async () => {
  try {
    const res = await getConfig()
    config.value = res.data || {}
  } catch (error) {
    message.error('加载配置失败: ' + error.message)
  }
}

const handleStart = async () => {
  loading.value.start = true
  try {
    await startService()
    message.success('服务启动成功')
    await fetchConfig()
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
    await fetchConfig()
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
    await fetchConfig()
  } catch (error) {
    message.error('重启失败: ' + error.message)
  } finally {
    loading.value.restart = false
  }
}

onMounted(fetchConfig)
</script>

<style scoped>
.settings h1 {
  margin-bottom: 24px;
  font-size: 28px;
  font-weight: 600;
}
</style>
