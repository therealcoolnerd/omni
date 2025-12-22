import React, { useState } from 'react';
import Sidebar from './components/Sidebar';
import Dashboard from './pages/Dashboard';
import './App.css';

function App() {
  const [activePage, setActivePage] = useState('dashboard');

  const renderContent = () => {
    switch (activePage) {
      case 'dashboard':
        return <Dashboard />;
      case 'packages':
        return <div className="p-8"><h1>Packages (Coming Soon)</h1></div>;
      case 'search':
        return <div className="p-8"><h1>Search (Coming Soon)</h1></div>;
      case 'settings':
        return <div className="p-8"><h1>Settings (Coming Soon)</h1></div>;
      default:
        return <Dashboard />;
    }
  };

  return (
    <div className="app-container">
      <Sidebar activePage={activePage} onNavigate={setActivePage} />
      <main className="main-content">
        {renderContent()}
      </main>
    </div>
  );
}

export default App;