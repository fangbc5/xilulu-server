<template>
  <div class="login-container">
    <!-- 背景装饰 -->
    <div class="background-decoration">
      <div class="gradient-orb orb-1"></div>
      <div class="gradient-orb orb-2"></div>
      <div class="gradient-orb orb-3"></div>
    </div>

    <!-- 登录卡片 -->
    <div class="login-card-wrapper">
      <n-card class="login-card" :bordered="false">
        <template #header>
          <div class="login-header">
            <div class="logo-container">
              <div class="logo-icon">
                <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                  <path
                    d="M12 2L2 7L12 12L22 7L12 2Z"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  />
                  <path
                    d="M2 17L12 22L22 17"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  />
                  <path
                    d="M2 12L12 17L22 12"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  />
                </svg>
              </div>
            </div>
            <h1 class="login-title">运营管理后台</h1>
            <p class="login-subtitle">Identity Service Admin Console</p>
          </div>
        </template>

        <n-form
          ref="formRef"
          :model="form"
          :rules="rules"
          size="large"
          :show-label="false"
          :show-require-mark="false"
        >
          <n-form-item path="username">
            <n-input
              v-model:value="form.username"
              placeholder="请输入用户名"
              :bordered="false"
              class="custom-input"
            >
              <template #prefix>
                <n-icon :component="UserIcon" :size="20" />
              </template>
            </n-input>
          </n-form-item>

          <n-form-item path="password">
            <n-input
              v-model:value="form.password"
              type="password"
              placeholder="请输入密码"
              :bordered="false"
              show-password-on="click"
              class="custom-input"
              @keyup.enter="handleLogin"
            >
              <template #prefix>
                <n-icon :component="LockIcon" :size="20" />
              </template>
            </n-input>
          </n-form-item>

          <n-form-item path="captcha">
            <div class="captcha-row">
              <n-input
                v-model:value="form.captcha"
                placeholder="请输入验证码"
                :bordered="false"
                class="custom-input captcha-input"
                @keyup.enter="handleLogin"
              >
                <template #prefix>
                  <n-icon :component="ShieldIcon" :size="20" />
                </template>
              </n-input>
              <div class="captcha-image">
                <img v-if="captchaImage" :src="captchaImage" alt="验证码" />
                <n-spin v-else :size="20" />
              </div>
              <div
                class="captcha-refresh-btn"
                :class="{ 'captcha-refresh-disabled': captchaCountdown > 0 }"
                @click="handleCaptchaClick"
                :title="captchaCountdown > 0 ? `${captchaCountdown}s 后可刷新` : '刷新验证码'"
              >
                <template v-if="captchaCountdown > 0">
                  {{ captchaCountdown }}s
                </template>
                <n-icon v-else :component="RefreshIcon" :size="16" />
              </div>
            </div>
          </n-form-item>

          <n-form-item>
            <n-button
              type="primary"
              block
              size="large"
              :loading="loading"
              @click="handleLogin"
              class="login-button"
            >
              {{ loading ? '登录中...' : '登录' }}
            </n-button>
          </n-form-item>
        </n-form>

        <div class="login-footer">
          <n-divider style="margin: 16px 0;">或</n-divider>
          <div class="login-tips">
            <n-text depth="3" class="text-sm">
              支持用户名/手机号/邮箱登录
            </n-text>
          </div>
        </div>
      </n-card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import {
  NCard,
  NForm,
  NFormItem,
  NInput,
  NButton,
  NIcon,
  NDivider,
  NText,
  NSpin,
  FormInst,
  useMessage
} from 'naive-ui';
import { PersonOutline as UserIcon, LockClosedOutline as LockIcon, ShieldCheckmarkOutline as ShieldIcon, RefreshOutline as RefreshIcon } from '@vicons/ionicons5';
import { useAuthStore } from '@/store/modules/auth';
import { handleApiResult } from '@/utils/request';
import { userApi } from '@/api/user';

const router = useRouter();
const route = useRoute();
const authStore = useAuthStore();
const message = useMessage();

const formRef = ref<FormInst | null>(null);
const loading = ref(false);
const captchaImage = ref<string>('');
const captchaId = ref<string>('');
const captchaCountdown = ref(0);
let countdownTimer: ReturnType<typeof setInterval> | null = null;

const form = ref({
  username: '',
  password: '',
  captcha: ''
});

const rules = {
  username: {
    required: true,
    message: '请输入用户名',
    trigger: 'blur'
  },
  password: {
    required: true,
    message: '请输入密码',
    trigger: 'blur'
  },
  captcha: {
    required: true,
    message: '请输入验证码',
    trigger: 'blur'
  }
};

const startCountdown = () => {
  captchaCountdown.value = 60;
  if (countdownTimer) clearInterval(countdownTimer);
  countdownTimer = setInterval(() => {
    captchaCountdown.value--;
    if (captchaCountdown.value <= 0) {
      if (countdownTimer) {
        clearInterval(countdownTimer);
        countdownTimer = null;
      }
    }
  }, 1000);
};

const refreshCaptcha = async () => {
  try {
    captchaImage.value = '';
    const response = await userApi.getCaptcha();
    const data = handleApiResult(response);
    if (data) {
      captchaId.value = data.captcha_id;
      captchaImage.value = `data:image/png;base64,${data.image_base64}`;
      startCountdown();
    }
  } catch (error: any) {
    // 如果是频率限制错误，也启动倒计时
    message.warning(error?.message || '获取验证码失败');
  }
};

const handleCaptchaClick = () => {
  if (captchaCountdown.value > 0) {
    message.warning(`请 ${captchaCountdown.value} 秒后再刷新验证码`);
    return;
  }
  refreshCaptcha();
};

onMounted(() => {
  refreshCaptcha();
});

onUnmounted(() => {
  if (countdownTimer) {
    clearInterval(countdownTimer);
    countdownTimer = null;
  }
});

const handleLogin = async () => {
  try {
    await formRef.value?.validate();
    loading.value = true;
    const success = await authStore.login({
      ...form.value,
      captcha_id: captchaId.value
    });
    if (success) {
      const redirect = (route.query.redirect as string) || '/';
      router.push(redirect);
    }
  } catch (error: any) {
    // 显示错误提示
    message.error(error?.message || '登录失败');
    // 刷新验证码
    refreshCaptcha();
    form.value.captcha = '';
  } finally {
    loading.value = false;
  }
};
</script>

<style scoped>
.login-container {
  position: relative;
  width: 100%;
  height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  overflow: hidden;
}

.background-decoration {
  position: absolute;
  width: 100%;
  height: 100%;
  top: 0;
  left: 0;
  overflow: hidden;
  z-index: 0;
}

.gradient-orb {
  position: absolute;
  border-radius: 50%;
  filter: blur(80px);
  opacity: 0.3;
  animation: float 20s infinite ease-in-out;
}

.orb-1 {
  width: 400px;
  height: 400px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  top: -200px;
  left: -200px;
  animation-delay: 0s;
}

.orb-2 {
  width: 300px;
  height: 300px;
  background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
  bottom: -150px;
  right: -150px;
  animation-delay: 5s;
}

.orb-3 {
  width: 350px;
  height: 350px;
  background: linear-gradient(135deg, #4facfe 0%, #00f2fe 100%);
  top: 50%;
  right: -175px;
  animation-delay: 10s;
}

@keyframes float {
  0%,
  100% {
    transform: translate(0, 0) scale(1);
  }
  33% {
    transform: translate(30px, -30px) scale(1.1);
  }
  66% {
    transform: translate(-20px, 20px) scale(0.9);
  }
}

.login-card-wrapper {
  position: relative;
  z-index: 1;
  width: 100%;
  max-width: 420px;
  padding: 20px;
}

.login-card {
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(20px);
  border-radius: 24px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  border: 1px solid rgba(255, 255, 255, 0.2);
  overflow: hidden;
}

.login-header {
  text-align: center;
  padding: 20px 0 10px;
}

.logo-container {
  display: flex;
  justify-content: center;
  margin-bottom: 16px;
}

.logo-icon {
  width: 64px;
  height: 64px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-radius: 16px;
  color: white;
  box-shadow: 0 8px 24px rgba(102, 126, 234, 0.4);
}

.logo-icon svg {
  width: 36px;
  height: 36px;
}

.login-title {
  font-size: 28px;
  font-weight: 700;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  margin: 0 0 8px 0;
  letter-spacing: -0.5px;
}

.login-subtitle {
  font-size: 14px;
  color: #8e8e93;
  margin: 0;
  font-weight: 400;
  letter-spacing: 0.5px;
}

.custom-input {
  background: #f5f5f7;
  border-radius: 12px;
  transition: all 0.3s ease;
}

.custom-input:hover {
  background: #ebebed;
}

.custom-input:focus-within {
  background: #ffffff;
  box-shadow: 0 0 0 2px rgba(102, 126, 234, 0.2);
}

.login-button {
  height: 48px;
  font-size: 16px;
  font-weight: 600;
  border-radius: 12px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border: none;
  box-shadow: 0 4px 16px rgba(102, 126, 234, 0.4);
  transition: all 0.3s ease;
}

.login-button:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(102, 126, 234, 0.5);
}

.login-button:active:not(:disabled) {
  transform: translateY(0);
}

.login-footer {
  margin-top: 8px;
}

.login-tips {
  text-align: center;
  padding: 8px 0;
}

.captcha-row {
  display: flex;
  width: 100%;
  gap: 12px;
  align-items: center;
}

.captcha-input {
  flex: 1;
}

.captcha-image {
  flex-shrink: 0;
  width: 110px;
  height: 42px;
  border-radius: 8px;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #f5f5f7;
}

.captcha-image img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.captcha-refresh-btn {
  flex-shrink: 0;
  width: 36px;
  height: 36px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #f5f5f7;
  cursor: pointer;
  color: #667eea;
  font-size: 12px;
  font-weight: 600;
  transition: all 0.3s ease;
}

.captcha-refresh-btn:hover:not(.captcha-refresh-disabled) {
  background: #ebebed;
  color: #764ba2;
}

.captcha-refresh-disabled {
  cursor: not-allowed;
  color: #8e8e93;
}

:deep(.n-input__input-el) {
  padding-left: 12px;
}

:deep(.n-input__prefix) {
  padding-left: 16px;
  color: #8e8e93;
}

:deep(.n-form-item) {
  margin-bottom: 20px;
}

:deep(.n-card__content) {
  padding: 24px 32px 32px;
}

:deep(.n-card__header) {
  padding: 32px 32px 0;
}
</style>

