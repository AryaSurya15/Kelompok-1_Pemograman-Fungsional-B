<script setup lang="ts">
import { onMounted, ref } from 'vue'
import AdminLayout from '../components/AdminLayout.vue'

interface Member {
  id: number
  name: string
  email: string
  joined_at: string  // dari backend berupa string datetime
}

const members = ref<Member[]>([])
const loading = ref(false)
const error = ref<string | null>(null)

// Form tambah member
const showAddForm = ref(false)
const newMemberName = ref('')
const newMemberEmail = ref('')
const saving = ref(false)

// Search sederhana (client-side)
const searchQuery = ref('')

// ambil data dari backend
async function loadMembers() {
  loading.value = true
  error.value = null
  try {
    const res = await fetch('http://127.0.0.1:8000/members')
    if (!res.ok) throw new Error(`HTTP ${res.status}`)
    const data: Member[] = await res.json()
    members.value = data
  } catch (e: any) {
    error.value = e.message ?? 'Gagal memuat data anggota'
  } finally {
    loading.value = false
  }
}

async function addMember() {
  if (!newMemberName.value.trim() || !newMemberEmail.value.trim()) {
    error.value = 'Nama dan email wajib diisi'
    return
  }

  saving.value = true
  error.value = null

  try {
    const res = await fetch('http://127.0.0.1:8000/members', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        name: newMemberName.value,
        email: newMemberEmail.value,
      }),
    })
    if (!res.ok) throw new Error(`HTTP ${res.status}`)
    const created: Member = await res.json()

    members.value.push(created)

    newMemberName.value = ''
    newMemberEmail.value = ''
    showAddForm.value = false
  } catch (e: any) {
    error.value = e.message ?? 'Gagal menambah anggota'
  } finally {
    saving.value = false
  }
}

async function deleteMember(id: number) {
  const ok = confirm(`Yakin hapus anggota dengan ID ${id}?`)
  if (!ok) return

  try {
    const res = await fetch(`http://127.0.0.1:8000/members/${id}`, {
      method: 'DELETE',
    })
    if (!res.ok) throw new Error(`HTTP ${res.status}`)
    const success: boolean = await res.json()
    if (success) {
      members.value = members.value.filter((m) => m.id !== id)
    } else {
      alert('Anggota tidak ditemukan atau gagal dihapus')
    }
  } catch (e: any) {
    error.value = e.message ?? 'Gagal menghapus anggota'
  }
}

function filteredMembers() {
  const q = searchQuery.value.toLowerCase()
  if (!q) return members.value
  return members.value.filter(
    (m) =>
      m.name.toLowerCase().includes(q) ||
      m.email.toLowerCase().includes(q),
  )
}

// Optional: format tanggal joined_at
function formatDate(dt: string): string {
  if (!dt) return '-'
  const d = new Date(dt)
  if (isNaN(d.getTime())) return dt
  return d.toLocaleString()
}

onMounted(() => {
  loadMembers()
})
</script>

<template>
  <AdminLayout title="Members">
    <div class="space-y-4">
      <!-- Header -->
      <div class="flex items-center justify-between">
        <div>
          <h2 class="text-lg font-semibold">Daftar Anggota</h2>
          <p class="text-xs text-gray-500">
            Data anggota yang dapat meminjam buku di Sudut Buku
          </p>
        </div>
        <button
          class="px-3 py-2 text-sm rounded bg-indigo-600 text-white hover:bg-indigo-700"
          @click="showAddForm = true"
        >
          + Tambah Anggota
        </button>
      </div>

      <!-- Search -->
      <section class="bg-white rounded border border-gray-200 p-3 space-y-2">
        <div class="flex flex-wrap items-center gap-2">
          <input
            v-model="searchQuery"
            type="text"
            placeholder="Cari nama atau email..."
            class="flex-1 min-w-[160px] border border-gray-300 rounded px-2 py-1 text-xs"
          />
          <span class="text-[10px] text-gray-500">
            Filter di sisi client (tanpa memanggil backend)
          </span>
        </div>
      </section>

      <!-- Error -->
      <p v-if="error" class="text-sm text-red-600">{{ error }}</p>

      <!-- Tabel -->
      <section class="bg-white rounded border border-gray-200 overflow-hidden">
        <div class="px-3 py-2 border-b border-gray-200 flex justify-between items-center">
          <span class="text-sm font-semibold">Data Anggota</span>
          <span class="text-xs text-gray-500">
            {{ filteredMembers().length }} anggota
          </span>
        </div>

        <table class="w-full border-collapse text-xs">
          <thead class="bg-gray-100 text-left">
            <tr>
              <th class="border p-2">ID</th>
              <th class="border p-2">Nama</th>
              <th class="border p-2">Email</th>
              <th class="border p-2">Bergabung</th>
              <th class="border p-2">Aksi</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="member in filteredMembers()
                "
              :key="member.id"
              class="hover:bg-gray-50"
            >
              <td class="border p-2">{{ member.id }}</td>
              <td class="border p-2 font-medium">{{ member.name }}</td>
              <td class="border p-2">{{ member.email }}</td>
              <td class="border p-2">{{ formatDate(member.joined_at) }}</td>
              <td class="border p-2">
                <button
                  class="px-2 py-1 text-[11px] rounded bg-red-600 text-white hover:bg-red-700"
                  @click="deleteMember(member.id)"
                >
                  Hapus
                </button>
              </td>
            </tr>

            <tr v-if="!loading && filteredMembers().length === 0">
              <td colspan="5" class="border p-2 text-center text-gray-500">
                Tidak ada anggota yang cocok.
              </td>
            </tr>
          </tbody>
        </table>
      </section>

      <!-- Form Tambah Anggota -->
      <div
        v-if="showAddForm"
        class="fixed inset-0 bg-black/40 flex items-center justify-center"
      >
        <div class="bg-white rounded shadow-lg w-full max-w-md p-4 space-y-3">
          <h3 class="text-sm font-semibold mb-1">Tambah Anggota Baru</h3>

          <div class="space-y-2 text-xs">
            <div>
              <label class="block mb-1">Nama</label>
              <input
                v-model="newMemberName"
                type="text"
                class="w-full border border-gray-300 rounded px-2 py-1"
              />
            </div>
            <div>
              <label class="block mb-1">Email</label>
              <input
                v-model="newMemberEmail"
                type="email"
                class="w-full border border-gray-300 rounded px-2 py-1"
              />
            </div>
          </div>

          <div class="flex justify-end gap-2 mt-3">
            <button
              class="px-3 py-1.5 text-xs rounded border border-gray-300"
              @click="showAddForm = false"
              :disabled="saving"
            >
              Batal
            </button>
            <button
              class="px-3 py-1.5 text-xs rounded bg-indigo-600 text-white hover:bg-indigo-700 disabled:opacity-50"
              @click="addMember"
              :disabled="saving"
            >
              {{ saving ? 'Menyimpan...' : 'Simpan' }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </AdminLayout>
</template>