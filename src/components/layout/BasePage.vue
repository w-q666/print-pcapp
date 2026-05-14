<script setup lang="ts">
import { TypographyTitle, Flex } from 'ant-design-vue'

defineProps<{
  title: string
}>()
</script>

<template>
  <div class="base-page">
    <Flex justify="space-between" align="center" class="base-page-header">
      <TypographyTitle :level="2" class="base-page-title">{{ title }}</TypographyTitle>
      <div class="base-page-actions">
        <slot name="actions" />
      </div>
    </Flex>
    <div class="base-page-content">
      <slot />
    </div>
  </div>
</template>
