
export interface Category {
  id: number;
  name: string;
  icon: string;
  count: number;
}

export interface Tool {
  id: number;
  name: string;
  description: string;
  icon: string;
  category?: Category; // 可选属性
  tags: string[];
  gradient: string;
}