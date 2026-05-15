import { defineStore } from 'pinia'
import { ref } from 'vue'

/** 系统配置页：顶栏分段与正文区共用当前选中的分区 key */
export const useSettingsNav = defineStore('settings-nav', () => {
  const activeKey = ref('fileFormat')
  return { activeKey }
})
