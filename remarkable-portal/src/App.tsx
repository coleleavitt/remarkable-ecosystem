import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { AuthProvider } from './contexts/AuthContext';
import { AppLayout } from './components/layout/Layout';
import { LoginPage } from './pages/LoginPage';
import { BrowserPage } from './pages/BrowserPage';
import { PreviewPage } from './pages/PreviewPage';
import { UploadPage } from './pages/UploadPage';
import { SharedPage } from './pages/SharedPage';
import { SettingsPage } from './pages/SettingsPage';
import { ShareViewPage } from './pages/ShareViewPage';

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 60, // 1 minute
      retry: 1,
    },
  },
});

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <AuthProvider>
        <BrowserRouter>
          <Routes>
            {/* Public routes */}
            <Route path="/login" element={<LoginPage />} />
            <Route path="/share/:token" element={<ShareViewPage />} />
            
            {/* Protected routes */}
            <Route element={<AppLayout />}>
              <Route path="/" element={<BrowserPage />} />
              <Route path="/folder/:folderId" element={<BrowserPage />} />
              <Route path="/recent" element={<BrowserPage />} />
              <Route path="/folders" element={<BrowserPage />} />
              <Route path="/preview/:documentId" element={<PreviewPage />} />
              <Route path="/upload" element={<UploadPage />} />
              <Route path="/shared" element={<SharedPage />} />
              <Route path="/settings" element={<SettingsPage />} />
            </Route>
          </Routes>
        </BrowserRouter>
      </AuthProvider>
    </QueryClientProvider>
  );
}

export default App;
