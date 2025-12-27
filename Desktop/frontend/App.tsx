import React from 'react';
import { HashRouter, Routes, Route } from 'react-router-dom';
import Layout from './components/Layout';
import ErrorBoundary from './components/ErrorBoundary';
import Dashboard from './pages/Dashboard';
import ReportManagement from './pages/ReportManagement';
import ReportDetail from './pages/ReportDetail';
import ComparisonAnalysis from './pages/ComparisonAnalysis';
import ThresholdConfig from './pages/ThresholdConfig';
import SqlAudit from './pages/SqlAudit';
import AuditLog from './pages/AuditLog';
import PlanVisualizer from './pages/PlanVisualizer';
import { I18nProvider } from './context/I18nContext';

const App: React.FC = () => {
  return (
    <ErrorBoundary>
      <I18nProvider>
        <HashRouter>
          <Layout>
            <Routes>
              <Route path="/" element={<Dashboard />} />
              <Route path="/reports" element={<ReportManagement />} />
              <Route path="/reports/:id" element={<ReportDetail />} />
              <Route path="/comparison" element={<ComparisonAnalysis />} />
              <Route path="/visualizer" element={<PlanVisualizer />} />
              <Route path="/thresholds" element={<ThresholdConfig />} />
              <Route path="/sqlaudit" element={<SqlAudit />} />
              <Route path="/auditlog" element={<AuditLog />} />
            </Routes>
          </Layout>
        </HashRouter>
      </I18nProvider>
    </ErrorBoundary>
  );
};

export default App;
