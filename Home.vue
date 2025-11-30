<template>
  <div>
    <h2>Home</h2>
    <button @click="fetchData">Coba Ambil Data dari Backend</button>
    <pre>{{ result }}</pre>
  </div>
</template>

<script setup>
import { ref } from 'vue'

const result = ref("Belum ada data")

async function fetchData() {
  try {
    const res = await fetch('http://localhost:8000/')
    result.value = await res.text()
  } catch (e) {
    result.value = 'Error: ' + e
  }
}
</script>

<style scoped>
button {
  padding: 8px 12px;
  border-radius: 6px;
  border: none;
  cursor: pointer;
}
</style>
