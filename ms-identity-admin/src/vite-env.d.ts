/// <reference types="vite/client" />

declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent<{}, {}, any>
  export default component
}

interface Window {
  $message?: any;
  $dialog?: any;
  $notification?: any;
  $loadingBar?: any;
}

