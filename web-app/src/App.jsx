import React, { useState } from 'react';
import TimelinePage from './pages/TimelinePage';
import ProfilePage from './pages/ProfilePage';
import './App.css';

/**
 * Main App Component with Security Headers and Routing
 */
function App() {
  const [currentPage, setCurrentPage] = useState('timeline');
  const [currentUserId, setCurrentUserId] = useState('user123');

  const renderPage = () => {
    switch (currentPage) {
      case 'profile':
        return <ProfilePage userId={currentUserId} />;
      case 'timeline':
      default:
        return <TimelinePage />;
    }
  };

  return (
    <div className="App">
      {/* Simple navigation for demo */}
      <nav className="bg-gray-800 border-b border-gray-700 p-4">
        <div className="max-w-4xl mx-auto flex gap-4">
          <button
            onClick={() => setCurrentPage('timeline')}
            className={`px-4 py-2 rounded-lg transition-colors ${
              currentPage === 'timeline'
                ? 'bg-omni-blue text-white'
                : 'bg-gray-700 text-white hover:bg-gray-600'
            }`}
          >
            Timeline
          </button>
          <button
            onClick={() => setCurrentPage('profile')}
            className={`px-4 py-2 rounded-lg transition-colors ${
              currentPage === 'profile'
                ? 'bg-omni-blue text-white'
                : 'bg-gray-700 text-white hover:bg-gray-600'
            }`}
          >
            Profile
          </button>
        </div>
      </nav>
      
      {/* Page content */}
      {renderPage()}
    </div>
  );
}

export default App;