import { create } from 'zustand';

interface CathedralStore {
  activeTab: string;
  setActiveTab: (tab: string) => void;
  engine: { status: string; throughput: number };
  setEngine: (e: any) => void;
  wormhole: { compressionRatio: number };
  setWormhole: (w: any) => void;
  depin: { activeWorkers: number; avgReputation: number };
  setDePIN: (d: any) => void;
}

export const useCathedralStore = create<CathedralStore>((set) => ({
  activeTab: 'dashboard',
  setActiveTab: (tab) => set({ activeTab: tab }),
  engine: { status: 'idle', throughput: 0 },
  setEngine: (e) => set((state) => ({ engine: { ...state.engine, ...e } })),
  wormhole: { compressionRatio: 0.95 },
  setWormhole: (w) => set((state) => ({ wormhole: { ...state.wormhole, ...w } })),
  depin: { activeWorkers: 0, avgReputation: 0 },
  setDePIN: (d) => set((state) => ({ depin: { ...state.depin, ...d } })),
}));
