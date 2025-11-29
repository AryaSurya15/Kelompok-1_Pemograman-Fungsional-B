<script setup lang="ts">
import { onMounted, ref, computed } from 'vue'
import AdminLayout from '../components/AdminLayout.vue'

interface Book {
  id: number
  available: boolean
}

interface Member {
  id: number
}

interface Loan {
  id: number
  due_at: string
  returned_at: string | null
}

const books = ref<Book[]>([])
const members = ref<Member[]>([])
const loans = ref<Loan[]>([])
const loading = ref(false)
const error = ref<string | null>(null)

async function loadSummary() {
  loading.value = true
  error.value = null
  try {
    const [bRes, mRes, lRes] = await Promise.all([
      fetch('http://127.0.0.1:8000/books'),
      fetch('http://127.0.0.1:8000/members'),
      fetch('http://127.0.0.1:8000/loans'),
    ])

    if (!bRes.ok) throw new Error('Gagal load books')
    if (!mRes.ok) throw new Error('Gagal load members')
    if (!lRes.ok) throw new Error('Gagal load loans')

    books.value = await bRes.json()
    members.value = await mRes.json()
    loans.value = await lRes.json()
  } catch (e: any) {
    error.value = e.message ?? 'Gagal memuat data ringkasan'
  } finally {
    loading.value = false
  }
}

const totalBooks = computed(() => books.value.length)
const totalMembers = computed(() => members.value.length)

const activeLoans = computed(() =>
  loans.value.filter((l) => !l.returned_at),
)

const activeLoansCount = computed(() => activeLoans.value.length)

const lateLoansCount = computed(() => {
  const now = new Date()
  return activeLoans.value.filter((l) => {
    const due = new Date(l.due_at)
    return now > due
  }).length
})

onMounted(() => {
  loadSummary()
})
</script>

<template>
  <AdminLayout title="Dashboard">
    <div class="space-y-4">
      <div class="flex items-center justify-between">
        <div>
          <h2 class="text-lg font-semibold">Ringkasan Sudut Buku</h2>
          <p class="text-xs text-gray-500">
            Statistik singkat koleksi, anggota, dan peminjaman.
          </p>
        </div>
        <button
          class="px-3 py-2 text-sm rounded bg-indigo-600 text-white hover:bg-indigo-700"
          @click="loadSummary"
        >
          Refresh
        </button>
      </div>

      <p v-if="error" class="text-sm text-red-600">{{ error }}</p>

      <!-- Cards -->
      <section class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <div class="bg-white rounded border border-gray-200 p-4">
          <p class="text-xs text-gray-500">Total Buku</p>
          <p class="text-2xl font-bold mt-1">{{ totalBooks }}</p>
        </div>

        <div class="bg-white rounded border border-gray-200 p-4">
          <p class="text-xs text-gray-500">Total Anggota</p>
          <p class="text-2xl font-bold mt-1">{{ totalMembers }}</p>
        </div>

        <div class="bg-white rounded border border-gray-200 p-4">
          <p class="text-xs text-gray-500">Peminjaman Aktif</p>
          <p class="text-2xl font-bold mt-1">{{ activeLoansCount }}</p>
        </div>

        <div class="bg-white rounded border border-gray-200 p-4">
          <p class="text-xs text-gray-500">Peminjaman Terlambat</p>
          <p class="text-2xl font-bold mt-1 text-red-600">
            {{ lateLoansCount }}
          </p>
        </div>
      </section>
    </div>
  </AdminLayout>
</template>
