
import React, { createContext, useState, useContext, ReactNode } from 'react';
import { WdrReportDetail, WdrObjectStat, RiskIssue } from '../types';

// ===== Comparison State (survives navigation away/back) =====
export interface ComparisonState {
  baseline: WdrReportDetail | null;
  targets: WdrReportDetail[];
  activeTab: 'metrics' | 'wait' | 'sql' | 'settings';
  sqlSortMode: 'total' | 'avg' | 'diff' | 'calls_diff';
  selectedCompSqlId: number | null;
  sqlUserFilter: string; // Initial value set to 'All' in Provider
}

interface WDRContextState {
  report: WdrReportDetail | null;
  setReport: (report: WdrReportDetail | null) => void;
  risks: RiskIssue[];
  setRisks: (risks: RiskIssue[]) => void;
  activeTab: 'overview' | 'wait' | 'sql' | 'obj' | 'settings';
  setActiveTab: (tab: 'overview' | 'wait' | 'sql' | 'obj' | 'settings') => void;
  selectedSql: any | null;
  setSelectedSql: (sql: any | null) => void;
  selectedObject: WdrObjectStat | null;
  setSelectedObject: (obj: WdrObjectStat | null) => void;
  objTypeFilter: 'All' | 'Table' | 'Index';
  setObjTypeFilter: (filter: 'All' | 'Table' | 'Index') => void;
  
  // New Filters
  sqlUserFilter: string;
  setSqlUserFilter: (user: string) => void;
  objSchemaFilter: string;
  setObjSchemaFilter: (schema: string) => void;

  reportHistory: WdrReportDetail[];
  setReportHistory: React.Dispatch<React.SetStateAction<WdrReportDetail[]>>;

  // Comparison state — persisted across navigation
  comparison: ComparisonState;
  setComparison: React.Dispatch<React.SetStateAction<ComparisonState>>;
}

const WDRContext = createContext<WDRContextState | undefined>(undefined);

export const WDRProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
  const [report, setReport] = useState<WdrReportDetail | null>(null);
  const [risks, setRisks] = useState<RiskIssue[]>([]);
  const [activeTab, setActiveTab] = useState<'overview' | 'wait' | 'sql' | 'obj' | 'settings'>('overview');
  const [selectedSql, setSelectedSql] = useState<any | null>(null);
  const [selectedObject, setSelectedObject] = useState<WdrObjectStat | null>(null);
  const [objTypeFilter, setObjTypeFilter] = useState<'All' | 'Table' | 'Index'>('Table'); 
  
  // New Filters State
  const [sqlUserFilter, setSqlUserFilter] = useState<string>('All');
  const [objSchemaFilter, setObjSchemaFilter] = useState<string>('All');

  const [reportHistory, setReportHistory] = useState<WdrReportDetail[]>([]);

  // Comparison state defaults
  const defaultComparison: ComparisonState = {
    baseline: null,
    targets: [],
    activeTab: 'metrics',
    sqlSortMode: 'total',
    selectedCompSqlId: null,
    sqlUserFilter: 'All',
  };

  const [comparison, setComparison] = useState<ComparisonState>(defaultComparison);

  return (
    <WDRContext.Provider value={{
      report, setReport,
      risks, setRisks,
      activeTab, setActiveTab,
      selectedSql, setSelectedSql,
      selectedObject, setSelectedObject,
      objTypeFilter, setObjTypeFilter,
      sqlUserFilter, setSqlUserFilter,
      objSchemaFilter, setObjSchemaFilter,
      reportHistory, setReportHistory,
      comparison, setComparison
    }}>
      {children}
    </WDRContext.Provider>
  );
};

export const useWDRContext = () => {
  const context = useContext(WDRContext);
  if (!context) {
    throw new Error('useWDRContext must be used within a WDRProvider');
  }
  return context;
};
