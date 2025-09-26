import { defineStore } from 'pinia'
import { Category, Tool } from "@/types/tools"


export const useToolStore = defineStore('tools', {
  state: () => ({
    categories: [] as Category[],
    tools: [] as Tool[],
    activeCategory: null as Category | null
  }),
  actions: {
    setCategories (categories: Category[]) {
      this.categories = categories
    },
    setTools (tools: Tool[]) {
      this.tools = tools
    },
    setActiveCategory (category: Category | null) {
      this.activeCategory = category
    }
  }
})