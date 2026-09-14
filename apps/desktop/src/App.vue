<script setup lang="ts">
import { onMounted } from 'vue'
import AppShell from '@/layouts/AppShell.vue'
import { useDeviceStore } from '@/stores/devices'

const devices = useDeviceStore()
onMounted(() => devices.load())
</script>

<template>
  <AppShell />
</template>
