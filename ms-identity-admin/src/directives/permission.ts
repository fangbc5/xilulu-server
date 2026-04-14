import type { App, DirectiveBinding } from 'vue';
import { watch } from 'vue';
import { usePermissionStore } from '@/store/modules/permission';

/**
 * v-permission 指令
 * 使用方式：
 *   <n-button v-permission="'admin:users:create'">新建用户</n-button>
 *   <n-button v-permission="['admin:users:create', 'admin:users:edit']">需要任一权限</n-button>
 *
 * 注意：需要在页面进入时调用 permissionStore.ensureMenuResources(...)
 * 预先加载当前菜单的权限数据。
 */
export function setupPermissionDirective(app: App) {
  app.directive('permission', {
    mounted(el: HTMLElement, binding: DirectiveBinding<string | string[]>) {
      const store = usePermissionStore();
      const getCodes = () => binding.value;

      const apply = () => {
        const allowed = store.hasPermission(getCodes());
        el.style.display = allowed ? '' : 'none';
      };

      // 初次挂载时根据当前权限设置显示/隐藏
      apply();

      // 监听当前菜单ID和权限code集合的变化（这是实际用于权限判断的数据）
      // 使用 nextTick 确保在数据恢复后也能触发更新
      watch(
        () => [store.currentMenuId, store.menuCodes],
        () => {
          // 使用 nextTick 确保响应式更新完成后再检查权限
          setTimeout(() => apply(), 0);
        },
        { deep: true, immediate: false }
      );
    },
    updated(el: HTMLElement, binding: DirectiveBinding<string | string[]>) {
      const store = usePermissionStore();
      const codes = binding.value;
      const allowed = store.hasPermission(codes);
      el.style.display = allowed ? '' : 'none';
    }
  });
}


