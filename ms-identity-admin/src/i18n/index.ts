import { createI18n } from 'vue-i18n';
import zhCN from './lang/zh-CN';
import enUS from './lang/en-US';

const i18n = createI18n({
  legacy: false,
  locale: 'zh-CN',
  fallbackLocale: 'en-US',
  messages: {
    'zh-CN': zhCN,
    'en-US': enUS
  }
});

export default i18n;

export const getMessage = (key: string): string => {
  return i18n.global.t(key) as string;
};

