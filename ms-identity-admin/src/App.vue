<template>
  <n-config-provider>
    <n-message-provider>
      <n-dialog-provider>
        <n-loading-bar-provider>
        <router-view />
        </n-loading-bar-provider>
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>

<script setup lang="ts">
import { NConfigProvider, NMessageProvider, NDialogProvider, NLoadingBarProvider } from 'naive-ui';
// App 根组件
</script>

<style>
#app {
  width: 100%;
  height: 100vh;
}
</style>

