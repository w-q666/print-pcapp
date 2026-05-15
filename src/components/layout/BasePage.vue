<script setup lang="ts">
import { TypographyTitle, Flex } from 'ant-design-vue'
import { usePlatform } from '../../composables/usePlatform'

const props = withDefaults(
  defineProps<{
    title: string
    /** page：整页滚动（默认）；content：仅内容区滚动，子页面需自行做内部 flex（如系统配置） */
    scrollMode?: 'page' | 'content'
  }>(),
  { scrollMode: 'page' },
)

const { platform } = usePlatform()
</script>

<template>
  <div
    class="base-page"
    :class="{
      'base-page--desktop-integrated': platform === 'desktop',
      'base-page--scroll-content': props.scrollMode === 'content',
    }"
  >
    <Flex
      v-if="platform !== 'desktop'"
      justify="space-between"
      align="center"
      class="base-page-header"
    >
      <TypographyTitle :level="2" class="base-page-title">{{ title }}</TypographyTitle>
      <div class="base-page-actions">
        <slot name="actions" />
      </div>
    </Flex>
    <Teleport v-if="platform === 'desktop'" to="#titlebar-page-actions">
      <div class="base-page-actions base-page-actions--titlebar">
        <slot name="actions" />
      </div>
    </Teleport>
    <div class="base-page-content">
      <slot />
    </div>
  </div>
</template>
