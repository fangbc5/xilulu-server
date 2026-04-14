<template>
  <div>
    <h1 class="text-2xl font-bold mb-4">用户管理</h1>
    <n-card>
      <template #header>
        <div class="flex justify-between items-center">
          <span>用户列表</span>
          <div class="flex gap-2">
            <n-input
              v-model:value="searchKey"
              placeholder="搜索用户名、邮箱、手机号、昵称"
              clearable
              style="width: 300px"
              @keyup.enter="handleSearch"
            />
            <n-button v-permission="'admin:users:search'" @click="handleSearch">搜索</n-button>
            <n-button v-permission="'admin:users:create'" type="primary" @click="handleCreate">创建用户</n-button>
          </div>
        </div>
      </template>
      <n-data-table
        :columns="columns"
        :data="users"
        :loading="loading"
        :pagination="pagination"
        remote
      />
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, h } from 'vue';
import { useRouter } from 'vue-router';
import {
  NCard,
  NButton,
  NDataTable,
  DataTableColumns,
  useDialog,
  useMessage,
  NButton as NButtonComponent,
  NInput,
  NIcon,
  NTag,
  NTooltip
} from 'naive-ui';
import { PencilOutline, TrashOutline } from '@vicons/ionicons5';
import { userApi } from '@/api/user';
import { handleApiResult } from '@/utils/request';
import { IUserInfo } from '@/types/base';
import { usePermissionStore } from '@/store/modules/permission';
import { useAuthStore } from '@/store/modules/auth';

const dialog = useDialog();
const message = useMessage();

const router = useRouter();
const loading = ref(false);
const users = ref<IUserInfo[]>([]);
const searchKey = ref('');
const permissionStore = usePermissionStore();
const authStore = useAuthStore();

const pagination = ref({
  page: 1,
  pageSize: 10,
  showSizePicker: true,
  pageSizes: [10, 20, 50],
  itemCount: 0,
  pageCount: 0,
  showQuickJumper: true,
  onChange: (page: number) => {
    pagination.value.page = page;
    loadUsers();
  },
  onUpdatePageSize: (pageSize: number) => {
    pagination.value.pageSize = pageSize;
    pagination.value.page = 1;
    loadUsers();
  },
  prefix: ({ itemCount }: { itemCount?: number }) => {
    const count = itemCount || 0;
    const totalPages = count > 0 ? Math.ceil(count / pagination.value.pageSize) : 0;
    return `共 ${count} 条，共 ${totalPages} 页`;
  }
});

const columns: DataTableColumns<IUserInfo> = [
  { title: 'ID', key: 'id', width: 80 },
  { title: '用户名', key: 'username', width: 120 },
  { title: '邮箱', key: 'email', width: 180 },
  { title: '手机号', key: 'mobile', width: 120 },
  { title: '昵称', key: 'nick_name', width: 120 },
  {
    title: '状态',
    key: 'state',
    width: 100,
    render: (row) => {
      return h(
        NTag,
        { type: row.state === 1 ? 'success' : 'error' },
        { default: () => (row.state === 1 ? '启用' : '禁用') }
      );
    }
  },
  {
    title: '操作',
    key: 'actions',
    width: 100,
    fixed: 'right',
    render: (row) => {
      const children: any[] = [];
      if (permissionStore.hasPermission('admin:users:edit')) {
        children.push(
          h(
            NTooltip,
            { trigger: 'hover', placement: 'top' },
            {
              trigger: () =>
                h(
                  NButtonComponent,
                  {
                    size: 'small',
                    type: 'primary',
                    onClick: () => handleEdit(row.id!),
                    quaternary: true,
                    circle: true
                  },
                  {
                    icon: () => h(NIcon, { component: PencilOutline, size: 16 })
                  }
                ),
              default: () => '编辑'
            }
          )
        );
      }
      if (permissionStore.hasPermission('admin:users:delete')) {
        children.push(
          h(
            NTooltip,
            { trigger: 'hover', placement: 'top' },
            {
              trigger: () =>
                h(
                  NButtonComponent,
                  {
                    size: 'small',
                    type: 'error',
                    onClick: () => handleDelete(row.id!),
                    quaternary: true,
                    circle: true
                  },
                  {
                    icon: () => h(NIcon, { component: TrashOutline, size: 16 })
                  }
                ),
              default: () => '删除'
            }
          )
        );
      }
      return h('div', { class: 'flex gap-1' }, children);
    }
  }
];

const loadUsers = async () => {
  loading.value = true;
  try {
    const response = await userApi.getUserList({
      page_size: pagination.value.pageSize,
      cursor: pagination.value.page > 1 ? pagination.value.page : undefined,
      search_key: searchKey.value || undefined
    });
    const data = handleApiResult(response);
    if (data) {
      users.value = data.list || [];
      const total = data.total || 0;
      pagination.value.itemCount = total;
      pagination.value.pageCount = total > 0 ? Math.ceil(total / pagination.value.pageSize) : 0;
    }
  } catch (error: any) {
    message.error(error.message || '加载用户列表失败');
  } finally {
    loading.value = false;
  }
};

const handleSearch = () => {
  pagination.value.page = 1;
  loadUsers();
};

const handleCreate = () => {
  router.push({ name: 'UserDetail', params: { id: 'new' } });
};

const handleEdit = (id: number) => {
  router.push({ name: 'UserDetail', params: { id } });
};

const handleDelete = async (id: number) => {
  dialog.warning({
    title: '确认删除',
    content: '确定要删除该用户吗？',
    positiveText: '确定',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        await userApi.deleteUser(id);
        message.success('删除成功');
        await loadUsers();
      } catch (error: any) {
        message.error(error.message || '删除失败');
      }
    }
  });
};

onMounted(async () => {
  await loadUsers();
  const menuId = permissionStore.currentMenuId;
  if (menuId) {
    // 确保菜单资源已加载（如果缓存中有则直接使用，没有则请求）
    await permissionStore.ensureMenuResources({
      application_id: 1, // TODO: 从配置或全局获取应用ID
      menu_id: menuId,
      tenant_id: authStore.user?.tenant_id
    });
    // 权限数据加载/恢复后，触发一次权限检查更新
    // 使用 nextTick 确保响应式更新完成
    await new Promise(resolve => setTimeout(resolve, 0));
  }
});
</script>

