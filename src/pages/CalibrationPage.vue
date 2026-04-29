<template>
  <div class="page calibration-page">
    <h2 class="page-title">模型校准</h2>
    <mdui-card variant="filled" class="form-card">
      <div class="form-content">
        <p class="section-desc">选择后验校准算法以优化浓度预测</p>

        <mdui-segmented-button-group
          v-model="calibrationModel"
          selects="single"
        >
          <mdui-segmented-button value="ekf">EKF</mdui-segmented-button>
          <mdui-segmented-button value="ou-kalman">OU-Kalman</mdui-segmented-button>
        </mdui-segmented-button-group>

        <div v-if="calibrationModel === 'ekf'" class="model-info">
          <h3>扩展卡尔曼滤波 (EKF)</h3>
          <p>基于贝叶斯推断的实时参数估计，通过观测值逐步修正模型参数。适用于有规律采血检测的场景。</p>
        </div>

        <div v-if="calibrationModel === 'ou-kalman'" class="model-info">
          <h3>OU-Kalman 滤波</h3>
          <p>基于 Ornstein-Uhlenbeck 过程的卡尔曼平滑器，结合前向滤波和后向平滑，提供更稳定的校准曲线。适用于稀疏观测数据。</p>
        </div>

        <mdui-button variant="filled" full-width @click="applyCalibration">
          应用校准
        </mdui-button>
      </div>
    </mdui-card>
  </div>
</template>

<script setup>
import { ref } from 'vue'

const calibrationModel = ref('ekf')

function applyCalibration() {
  console.log('Calibration model applied:', calibrationModel.value)
}
</script>

<style scoped>
.calibration-page {
  max-width: 600px;
  margin: 0 auto;
}

.page-title {
  font-size: 24px;
  font-weight: 600;
  margin-bottom: 16px;
}

.form-card {
  padding: 16px;
}

.form-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.section-desc {
  opacity: 0.7;
  font-size: 14px;
}

.model-info {
  padding: 16px;
  border-radius: 12px;
  background: var(--mdui-color-surface-container);
}

.model-info h3 {
  font-size: 16px;
  font-weight: 600;
  margin-bottom: 8px;
}

.model-info p {
  font-size: 13px;
  line-height: 1.6;
  opacity: 0.8;
}
</style>
