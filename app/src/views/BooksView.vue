<script setup lang="ts">
import { onMounted, ref } from 'vue'
import AdminLayout from '../components/AdminLayout.vue'

interface Book {
  id: number
  title: string
  author: string
  category: string
  year: number
  total_copies: number
  available_copies: number
}

const books = ref<Book[]>([])
const filteredBooks = ref<Book[]>([])
const loading = ref(false)
const error = ref<string | null>(null)

// Search
const searchQuery = ref('')
const searchMode = ref<'title' | 'author' | 'category'>('title')
const searching = ref(false)

// Form tambah buku
const showAddForm = ref(false)
const newBookTitle = ref('')
const newBookAuthor = ref('')
const newBookCategory = ref('')
const newBookYear = ref<number | null>(null)
const newBookCopies = ref<number | null>(1)
const saving = ref(false)

async function loadBooks() {
  loading.value = true
  error.value = null
  try {
    const res = await fetch('http://127.0.0.1:8000/books')
    if (!res.ok) throw new Error(`HTTP ${res.status}`)
    const data: Book[] = await res.json()
    books.value = data
    filteredBooks.value = data
  } catch (e: any) {
    error.value = e.message ?? 'Gagal memuat data buku'
  } finally {
    loading.value = false
  }
}

async function runSearch() {
  if (!searchQuery.value.trim()) {
    filteredBooks.value = books.value
    return
  }
  searching.value = true
  error.value = null
  try {
    const params = new URLSearchParams({
      q: searchQuery.value,
      mode: searchMode.value,
    })
    const url = `http://127.0.0.1:8000/search?${params.toString()}`
    const res = await fetch(url)
    if (!res.ok) throw new Error(`HTTP ${res.status}`)
    const data: Book[] = await res.json()
    filteredBooks.value = data
  } catch (e: any) {
    error.value = e.message ?? 'Gagal melakukan pencarian'
  } finally {
    searching.value = false
  }
}

async function addBook() {
  if (
    !newBookTitle.value.trim() ||
    !newBookAuthor.value.trim() ||
    !newBookCategory.value.trim() ||
    !newBookYear.value ||
    !newBookCopies.value ||
    newBookCopies.value <= 0
  ) {
    error.value = 'Semua field wajib diisi dan stok > 0'
    return
  }

  saving.value = true
  error.value = null
  try {
    const res = await fetch('http://127.0.0.1:8000/books', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        title: newBookTitle.value,
        author: newBookAuthor.value,
        category: newBookCategory.value,
        year: newBookYear.value,
        total_copies: newBookCopies.value,
      }),
    })
    if (!res.ok) throw new Error(`HTTP ${res.status}`)
    const created: Book = await res.json()

    books.value.push(created)
    filteredBooks.value = books.value

    newBookTitle.value = ''
    newBookAuthor.value = ''
    newBookCategory.value = ''
    newBookYear.value = null
    newBookCopies.value = 1
    showAddForm.value = false
  } catch (e: any) {
    error.value = e.message ?? 'Gagal menambah buku'
  } finally {
    saving.value = false
  }
}

async function deleteBook(id: number) {
  const ok = confirm(`Yakin hapus buku dengan ID ${id}?`)
  if (!ok) return

  try {
    const res = await fetch(`http://127.0.0.1:8000/books/${id}`, {
      method: 'DELETE',
    })
    if (!res.ok) throw new Error(`HTTP ${res.status}`)
    const success: boolean = await res.json()
    if (success) {
      books.value = books.value.filter((b) => b.id !== id)
      filteredBooks.value = filteredBooks.value.filter((b) => b.id !== id)
    } else {
      alert('Buku tidak ditemukan atau gagal dihapus')
    }
  } catch (e: any) {
    error.value = e.message ?? 'Gagal menghapus buku'
  }
}

onMounted(() => {
  loadBooks()
})
</script>

<template>
  <AdminLayout title="Books">
    <div class="space-y-4">
      <!-- Header -->
      <div class="flex items-center justify-between">
        <div>
          <h2 class="text-lg font-semibold">Daftar Buku</h2>
          <p class="text-xs text-gray-500">
            Kelola koleksi buku Sudut Buku (data dari MySQL via Axum)
          </p>
        </div>
        <button
          class="px-3 py-2 text-sm rounded bg-indigo-600 text-white hover:bg-indigo-700"
          @click="showAddForm = true"
        >
          + Tambah Buku
        </button>
      </div>

      <!-- Search -->
      <section class="bg-white rounded border border-gray-200 p-3 space-y-2">
        <div class="flex flex-wrap gap-2 items-center">
          <label class="text-xs text-gray-600">Cari berdasarkan:</label>
          <select
            v-model="searchMode"
            class="border border-gray-300 rounded px-2 py-1 text-xs"
          >
            <option value="title">Judul</option>
            <option value="author">Penulis</option>
            <option value="category">Kategori</option>
          </select>

          <input
            v-model="searchQuery"
            type="text"
            class="flex-1 min-w-[160px] border border-gray-300 rounded px-2 py-1 text-xs"
            placeholder="Ketik kata kunci..."
            @keyup.enter="runSearch"
          />

          <button
            class="px-3 py-1.5 text-xs rounded bg-emerald-600 text-white hover:bg-emerald-700"
            @click="runSearch"
          >
            Cari
          </button>

          <span v-if="searching" class="text-[10px] text-gray-500">
            Mencari (parallel)...
          </span>
        </div>

        <p class="text-[10px] text-gray-400">
          Endpoint: <code>/search?mode={{ searchMode }}&amp;q={{ searchQuery }}</code>
        </p>
      </section>

      <!-- Error -->
      <p v-if="error" class="text-sm text-red-600">{{ error }}</p>

      <!-- Tabel -->
      <section class="bg-white rounded border border-gray-200 overflow-hidden">
        <div class="px-3 py-2 border-b border-gray-200 flex justify-between items-center">
          <span class="text-sm font-semibold">Data Buku</span>
          <span class="text-xs text-gray-500">
            {{ filteredBooks.length }} / {{ books.length }} buku
          </span>
        </div>

        <table class="w-full border-collapse text-xs">
          <thead class="bg-gray-100 text-left">
            <tr>
              <th class="border p-2">ID</th>
              <th class="border p-2">Judul</th>
              <th class="border p-2">Penulis</th>
              <th class="border p-2">Kategori</th>
              <th class="border p-2">Tahun</th>
              <th class="border p-2">Stok</th>
              <th class="border p-2">Aksi</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="book in filteredBooks"
              :key="book.id"
              class="hover:bg-gray-50"
            >
              <td class="border p-2">{{ book.id }}</td>
              <td class="border p-2 font-medium">{{ book.title }}</td>
              <td class="border p-2">{{ book.author }}</td>
              <td class="border p-2">{{ book.category }}</td>
              <td class="border p-2">{{ book.year }}</td>
              <td class="border p-2">
                <span
                  class="px-2 py-0.5 rounded-full text-[11px] font-semibold"
                  :class="book.available_copies > 0
                    ? 'bg-emerald-100 text-emerald-700'
                    : 'bg-red-100 text-red-700'"
                >
                  <!-- kalau ada stok, tampilkan sisa/total, kalau tidak 'Stok habis' -->
                  {{ book.available_copies > 0
                    ? `Ready (${book.available_copies}/${book.total_copies})`
                    : 'Stok habis' }}
                </span>
              </td>
              <td class="border p-2">
                <button
                  class="px-2 py-1 text-[11px] rounded bg-red-600 text-white hover:bg-red-700"
                  @click="deleteBook(book.id)"
                >
                  Hapus
                </button>
              </td>
            </tr>

            <tr v-if="!loading && filteredBooks.length === 0">
              <td colspan="7" class="border p-2 text-center text-gray-500">
                Tidak ada buku yang cocok.
              </td>
            </tr>
          </tbody>
        </table>
      </section>

      <!-- Form Tambah Buku (modal sederhana) -->
      <div
        v-if="showAddForm"
        class="fixed inset-0 bg-black/40 flex items-center justify-center"
      >
        <div class="bg-white rounded shadow-lg w-full max-w-md p-4 space-y-3">
          <h3 class="text-sm font-semibold mb-1">Tambah Buku Baru</h3>

          <div class="space-y-2 text-xs">
            <div>
              <label class="block mb-1">Judul</label>
              <input
                v-model="newBookTitle"
                type="text"
                class="w-full border border-gray-300 rounded px-2 py-1"
              />
            </div>
            <div>
              <label class="block mb-1">Penulis</label>
              <input
                v-model="newBookAuthor"
                type="text"
                class="w-full border border-gray-300 rounded px-2 py-1"
              />
            </div>
            <div>
              <label class="block mb-1">Kategori</label>
              <input
                v-model="newBookCategory"
                type="text"
                class="w-full border border-gray-300 rounded px-2 py-1"
              />
            </div>
            <div>
              <label class="block mb-1">Tahun</label>
              <input
                v-model.number="newBookYear"
                type="number"
                class="w-full border border-gray-300 rounded px-2 py-1"
              />
            </div>
            <div>
              <label class="block mb-1">Jumlah Eksemplar</label>
              <input
                v-model.number="newBookCopies"
                type="number"
                min="1"
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
              @click="addBook"
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