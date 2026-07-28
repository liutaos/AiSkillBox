<template>
  <div class="skills">
    <h1>Skill管理</h1>

    <n-card>
      <n-space>
        <n-input
          v-model:value="searchQuery"
          placeholder="搜索skill名称或描述"
          clearable
          style="width: 300px"
          @keyup.enter="handleSearch"
        />
        <n-input
          v-model:value="searchTags"
          placeholder="标签筛选"
          clearable
          style="width: 200px"
          @keyup.enter="handleSearch"
        />
        <n-button
          type="primary"
          @click="handleSearch"
          :loading="loading.search"
        >
          搜索
        </n-button>
        <n-button @click="handleReset"> 重置 </n-button>
      </n-space>
    </n-card>

    <n-card style="margin-top: 16px">
      <n-data-table
        :columns="columns"
        :data="skills"
        :loading="loading.table"
        :pagination="pagination"
        :row-key="(row) => row.name"
        remote
      />
    </n-card>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted, h } from "vue";
import { NButton, NSwitch, NSpace, NPopconfirm, useMessage } from "naive-ui";
import {
  getSkills,
  searchSkills,
  deleteSkill,
  enableSkill,
  disableSkill,
} from "../api";

const message = useMessage();

const skills = ref([]);
const searchQuery = ref("");
const searchTags = ref("");
const currentPage = ref(1);
const pageSize = ref(20);
const total = ref(0);

const loading = reactive({
  table: false,
  search: false,
});

const pagination = reactive({
  page: currentPage.value,
  pageSize: pageSize.value,
  pageSizes: [10, 20, 50],
  showSizePicker: true,
  itemCount: total.value,
  onChange: (page) => {
    currentPage.value = page;
    pagination.page = page;
    fetchSkills();
  },
  onUpdatePageSize: (size) => {
    pageSize.value = size;
    currentPage.value = 1;
    pagination.page = 1;
    pagination.pageSize = size;
    fetchSkills();
  },
});

const columns = [
  {
    title: "名称",
    key: "name",
    width: 200,
  },
  {
    title: "描述",
    key: "description",
    ellipsis: { tooltip: true },
  },
  {
    title: "标签",
    key: "tags",
    width: 200,
    render(row) {
      const tags = JSON.parse(row.tags || "[]");
      return h(
        NSpace,
        { size: 4 },
        {
          default: () =>
            tags.map((tag) =>
              h(
                NButton,
                { size: "small", secondary: true },
                { default: () => tag },
              ),
            ),
        },
      );
    },
  },
  {
    title: "状态",
    key: "enabled",
    width: 100,
    render(row) {
      return h(NSwitch, {
        value: row.enabled,
        onUpdateValue: (value) => handleToggle(row, value),
      });
    },
  },
  {
    title: "操作",
    key: "actions",
    width: 100,
    render(row) {
      return h(
        NSpace,
        { size: 4 },
        {
          default: () => [
            h(
              NPopconfirm,
              {
                onPositiveClick: () => handleDelete(row),
              },
              {
                trigger: () =>
                  h(
                    NButton,
                    { size: "small", type: "error" },
                    { default: () => "删除" },
                  ),
                default: () => `确定删除 ${row.name} 吗？`,
              },
            ),
          ],
        },
      );
    },
  },
];

const fetchSkills = async () => {
  loading.table = true;
  try {
    const res = await getSkills({
      page: currentPage.value,
      size: pageSize.value,
    });
    skills.value = res.data?.skills || [];
    total.value = res.data?.count || 0;
    pagination.itemCount = total.value;
  } catch (error) {
    message.error("加载失败: " + error.message);
  } finally {
    loading.table = false;
  }
};

const handleSearch = async () => {
  currentPage.value = 1;
  pagination.page = 1;

  if (!searchQuery.value && !searchTags.value) {
    return fetchSkills();
  }

  loading.search = true;
  try {
    const res = await searchSkills(searchQuery.value, searchTags.value);
    skills.value = res.data?.skills || [];
    total.value = res.data?.count || 0;
    pagination.itemCount = total.value;
  } catch (error) {
    message.error("搜索失败: " + error.message);
  } finally {
    loading.search = false;
  }
};

const handleReset = () => {
  searchQuery.value = "";
  searchTags.value = "";
  currentPage.value = 1;
  pagination.page = 1;
  fetchSkills();
};

const handleToggle = async (row, value) => {
  try {
    if (value) {
      await enableSkill(row.name);
      message.success(`${row.name} 已启用`);
    } else {
      await disableSkill(row.name);
      message.success(`${row.name} 已禁用`);
    }
    row.enabled = value;
  } catch (error) {
    message.error("操作失败: " + error.message);
  }
};

const handleDelete = async (row) => {
  try {
    await deleteSkill(row.name);
    message.success(`${row.name} 已删除`);
    fetchSkills();
  } catch (error) {
    message.error("删除失败: " + error.message);
  }
};

onMounted(fetchSkills);
</script>

<style scoped>
.skills h1 {
  margin-bottom: 24px;
  font-size: 28px;
  font-weight: 600;
}
</style>
