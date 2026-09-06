<template>
  <div class="memory-page">
    <h1>记忆/问题管理</h1>
    
    <!-- 统计卡片 -->
    <n-grid :cols="4" :x-gap="16" style="margin-bottom: 24px">
      <n-gi>
        <n-card class="stat-card">
          <n-statistic label="记忆总数" :value="memStats.total" />
        </n-card>
      </n-gi>
      <n-gi>
        <n-card class="stat-card">
          <n-statistic label="活跃记忆" :value="memStats.active" />
        </n-card>
      </n-gi>
      <n-gi>
        <n-card class="stat-card">
          <n-statistic label="已归档" :value="memStats.archived" />
        </n-card>
      </n-gi>
      <n-gi>
        <n-card class="stat-card">
          <n-statistic label="问题总数" :value="issueStats.total" />
        </n-card>
      </n-gi>
    </n-grid>
    
    <!-- 记忆管理 -->
    <n-card title="记忆管理" style="margin-bottom: 24px">
      <template #header-extra>
        <n-space>
          <n-switch v-model:value="showArchived" @update:value="loadMemories">
            <template #checked>显示归档</template>
            <template #unchecked>隐藏归档</template>
          </n-switch>
          <n-input v-model:value="memSearchQuery" placeholder="搜索记忆..." clearable style="width: 200px" @keyup.enter="searchMemories" />
          <n-button type="primary" @click="searchMemories" :loading="memLoading.search">搜索</n-button>
          <n-button @click="resetMemSearch">重置</n-button>
        </n-space>
      </template>
      
      <n-data-table
        :columns="memColumns"
        :data="memories"
        :loading="memLoading.table"
        :pagination="memPagination"
        :row-key="row => row.id"
        remote
      />
    </n-card>
    
    <!-- 问题库 -->
    <n-card title="问题库">
      <template #header-extra>
        <n-space>
          <n-input v-model:value="issueSearchQuery" placeholder="搜索问题..." clearable style="width: 200px" @keyup.enter="searchIssues" />
          <n-select v-model:value="issueCategoryFilter" placeholder="分类" clearable style="width: 120px" :options="categoryOptions" @update:value="searchIssues" />
          <n-select v-model:value="issueTaskFilter" placeholder="任务" clearable style="width: 120px" :options="taskOptions" @update:value="searchIssues" />
          <n-button type="primary" @click="searchIssues" :loading="issueLoading.search">搜索</n-button>
          <n-button @click="resetIssueSearch">重置</n-button>
        </n-space>
      </template>
      
      <n-data-table
        :columns="issueColumns"
        :data="issues"
        :loading="issueLoading.table"
        :pagination="issuePagination"
        :row-key="row => row.id"
        remote
      />
    </n-card>
  </div>
</template>

<script setup>
import { ref, reactive, h, onMounted } from 'vue'
import { useMessage, NButton, NTag, NSpace, NPopconfirm } from 'naive-ui'
import { 
  getMemories, 
  searchMemories as apiSearchMemories, 
  deleteMemory,
  getMemoryStats,
  getIssues,
  searchIssues as apiSearchIssues,
  deleteIssue
} from '../api'

const message = useMessage()

const memStats = ref({ total: 0, active: 0, archived: 0 })
const issueStats = ref({ total: 0 })

const memories = ref([])
const memSearchQuery = ref('')
const memCurrentPage = ref(1)
const memPageSize = ref(20)
const memTotal = ref(0)
const memLoading = reactive({ table: false, search: false })
const showArchived = ref(false)

const memPagination = reactive({
  page: memCurrentPage.value,
  pageSize: memPageSize.value,
  pageSizes: [10, 20, 50],
  showSizePicker: true,
  itemCount: memTotal.value,
  onChange: (page) => {
    memCurrentPage.value = page
    memPagination.page = page
    loadMemories()
  },
  onUpdatePageSize: (size) => {
    memPageSize.value = size
    memCurrentPage.value = 1
    memPagination.page = 1
    memPagination.pageSize = size
    loadMemories()
  }
})

const issues = ref([])
const issueSearchQuery = ref('')
const issueCategoryFilter = ref(null)
const issueTaskFilter = ref(null)
const issueCurrentPage = ref(1)
const issuePageSize = ref(20)
const issueTotal = ref(0)
const issueLoading = reactive({ table: false, search: false })

const issuePagination = reactive({
  page: issueCurrentPage.value,
  pageSize: issuePageSize.value,
  pageSizes: [10, 20, 50],
  showSizePicker: true,
  itemCount: issueTotal.value,
  onChange: (page) => {
    issueCurrentPage.value = page
    issuePagination.page = page
    loadIssues()
  },
  onUpdatePageSize: (size) => {
    issuePageSize.value = size
    issueCurrentPage.value = 1
    issuePagination.page = 1
    issuePagination.pageSize = size
    loadIssues()
  }
})

const categoryOptions = [
  { label: '安装', value: '安装' },
  { label: '配置', value: '配置' },
  { label: '运行', value: '运行' },
  { label: '兼容性', value: '兼容性' },
  { label: '性能', value: '性能' },
  { label: '功能', value: '功能' }
]

const taskOptions = [
  { label: 'skill管理', value: 'skill管理' },
  { label: 'MCP协议', value: 'MCP协议' },
  { label: '搜索', value: '搜索' },
  { label: '导入', value: '导入' },
  { label: 'UI界面', value: 'UI界面' },
  { label: '数据库', value: '数据库' }
]

const memColumns = [
  { title: 'ID', key: 'id', width: 60 },
  { title: '内容', key: 'content', ellipsis: { tooltip: true } },
  { title: '标签', key: 'tags', width: 120 },
  { title: '来源', key: 'source', width: 100 },
  { title: '状态', key: 'is_archived', width: 80, render(row) {
    return h(NTag, { type: row.is_archived ? 'warning' : 'success', size: 'small' }, () => row.is_archived ? '已归档' : '活跃')
  }},
  { title: '访问次数', key: 'access_count', width: 80 },
  { title: '创建时间', key: 'created_at', width: 160 },
  {
    title: '操作',
    key: 'actions',
    width: 100,
    render(row) {
      return h(NSpace, { size: 4 }, () => [
        h(NPopconfirm, {
          onPositiveClick: () => handleDeleteMemory(row.id)
        }, {
          trigger: () => h(NButton, { size: 'small', type: 'error' }, () => '删除'),
          default: () => `确定删除这条记忆吗？`
        })
      ])
    }
  }
]

const issueColumns = [
  { title: 'ID', key: 'id', width: 60 },
  { title: '分类', key: 'category', width: 80, render(row) { return h(NTag, { type: 'warning', size: 'small' }, () => row.category) } },
  { title: '任务', key: 'task', width: 100 },
  { title: '场景', key: 'scenario', ellipsis: { tooltip: true } },
  { title: '解决方案', key: 'solution', ellipsis: { tooltip: true } },
  { title: '频率', key: 'frequency', width: 60 },
  {
    title: '操作',
    key: 'actions',
    width: 100,
    render(row) {
      return h(NSpace, { size: 4 }, () => [
        h(NPopconfirm, {
          onPositiveClick: () => handleDeleteIssue(row.id)
        }, {
          trigger: () => h(NButton, { size: 'small', type: 'error' }, () => '删除'),
          default: () => `确定删除这个问题吗？`
        })
      ])
    }
  }
]

const loadMemStats = async () => {
  try {
    const res = await getMemoryStats()
    memStats.value = res.data || {}
  } catch (e) {
    console.error('加载记忆统计失败:', e)
  }
}

const loadMemories = async () => {
  memLoading.table = true
  try {
    const res = await getMemories({ page: memCurrentPage.value, size: memPageSize.value, include_archived: showArchived.value })
    memories.value = res.data?.memories || []
    memTotal.value = res.data?.count || 0
    memPagination.itemCount = memTotal.value
  } catch (e) {
    message.error('加载记忆失败: ' + e.message)
  } finally {
    memLoading.table = false
  }
}

const searchMemories = async () => {
  if (!memSearchQuery.value) {
    await loadMemories()
    return
  }
  memLoading.search = true
  memCurrentPage.value = 1
  memPagination.page = 1
  try {
    const res = await apiSearchMemories({ query: memSearchQuery.value, limit: 100 })
    memories.value = res.data?.memories || []
    memTotal.value = res.data?.count || 0
    memPagination.itemCount = memTotal.value
  } catch (e) {
    message.error('搜索失败: ' + e.message)
  } finally {
    memLoading.search = false
  }
}

const resetMemSearch = () => {
  memSearchQuery.value = ''
  memCurrentPage.value = 1
  memPagination.page = 1
  loadMemories()
}

const handleDeleteMemory = async (id) => {
  try {
    await deleteMemory(id)
    message.success('删除成功')
    await loadMemories()
    await loadMemStats()
  } catch (e) {
    message.error('删除失败: ' + e.message)
  }
}

const loadIssues = async () => {
  issueLoading.table = true
  try {
    const res = await getIssues({ page: issueCurrentPage.value, size: issuePageSize.value })
    issues.value = res.data?.issues || []
    issueTotal.value = res.data?.count || 0
    issuePagination.itemCount = issueTotal.value
    issueStats.value.total = issueTotal.value
  } catch (e) {
    message.error('加载问题失败: ' + e.message)
  } finally {
    issueLoading.table = false
  }
}

const searchIssues = async () => {
  if (!issueSearchQuery.value && !issueCategoryFilter.value && !issueTaskFilter.value) {
    await loadIssues()
    return
  }
  issueLoading.search = true
  issueCurrentPage.value = 1
  issuePagination.page = 1
  try {
    const res = await apiSearchIssues({
      query: issueSearchQuery.value || '',
      category: issueCategoryFilter.value || undefined,
      task: issueTaskFilter.value || undefined,
      limit: 100
    })
    issues.value = res.data?.issues || []
    issueTotal.value = res.data?.count || 0
    issuePagination.itemCount = issueTotal.value
  } catch (e) {
    message.error('搜索失败: ' + e.message)
  } finally {
    issueLoading.search = false
  }
}

const resetIssueSearch = () => {
  issueSearchQuery.value = ''
  issueCategoryFilter.value = null
  issueTaskFilter.value = null
  issueCurrentPage.value = 1
  issuePagination.page = 1
  loadIssues()
}

const handleDeleteIssue = async (id) => {
  try {
    await deleteIssue(id)
    message.success('删除成功')
    await loadIssues()
  } catch (e) {
    message.error('删除失败: ' + e.message)
  }
}

onMounted(async () => {
  await Promise.all([loadMemStats(), loadMemories(), loadIssues()])
  issueStats.value.total = issueTotal.value
})
</script>

<style scoped>
.memory-page h1 {
  margin-bottom: 24px;
  font-size: 28px;
  font-weight: 600;
}

.stat-card {
  text-align: center;
}
</style>
