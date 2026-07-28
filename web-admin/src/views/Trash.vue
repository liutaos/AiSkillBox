<template>
  <div class="trash">
    <h1>回收站</h1>

    <n-card>
      <n-data-table
        :columns="columns"
        :data="trashList"
        :loading="loading"
        :pagination="pagination"
        :row-key="(row) => row.name"
        remote
      />
    </n-card>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted, h } from "vue";
import { NButton, NSpace, NPopconfirm, useMessage } from "naive-ui";
import { getTrash, restoreSkill, permanentDelete } from "../api";

const message = useMessage();

const trashList = ref([]);
const loading = ref(false);
const currentPage = ref(1);
const pageSize = ref(20);
const total = ref(0);

const pagination = reactive({
  page: currentPage.value,
  pageSize: pageSize.value,
  pageSizes: [10, 20, 50],
  showSizePicker: true,
  itemCount: total.value,
  onChange: (page) => {
    currentPage.value = page;
    pagination.page = page;
    fetchTrash();
  },
  onUpdatePageSize: (size) => {
    pageSize.value = size;
    currentPage.value = 1;
    pagination.page = 1;
    pagination.pageSize = size;
    fetchTrash();
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
    title: "删除时间",
    key: "updated_at",
    width: 180,
  },
  {
    title: "操作",
    key: "actions",
    width: 180,
    render(row) {
      return h(
        NSpace,
        { size: 4 },
        {
          default: () => [
            h(
              NButton,
              {
                size: "small",
                type: "success",
                onClick: () => handleRestore(row),
              },
              { default: () => "恢复" },
            ),
            h(
              NPopconfirm,
              {
                onPositiveClick: () => handlePermanentDelete(row),
              },
              {
                trigger: () =>
                  h(
                    NButton,
                    { size: "small", type: "error" },
                    { default: () => "永久删除" },
                  ),
                default: () => `确定永久删除 ${row.name} 吗？此操作不可恢复！`,
              },
            ),
          ],
        },
      );
    },
  },
];

const fetchTrash = async () => {
  loading.value = true;
  try {
    const res = await getTrash({
      page: currentPage.value,
      size: pageSize.value,
    });
    trashList.value = res.data?.skills || [];
    total.value = res.data?.count || 0;
    pagination.itemCount = total.value;
  } catch (error) {
    message.error("加载失败: " + error.message);
  } finally {
    loading.value = false;
  }
};

const handleRestore = async (row) => {
  try {
    await restoreSkill(row.name);
    message.success(`${row.name} 已恢复`);
    fetchTrash();
  } catch (error) {
    message.error("恢复失败: " + error.message);
  }
};

const handlePermanentDelete = async (row) => {
  try {
    await permanentDelete(row.name);
    message.success(`${row.name} 已永久删除`);
    fetchTrash();
  } catch (error) {
    message.error("删除失败: " + error.message);
  }
};

onMounted(fetchTrash);
</script>

<style scoped>
.trash h1 {
  margin-bottom: 24px;
  font-size: 28px;
  font-weight: 600;
}
</style>
