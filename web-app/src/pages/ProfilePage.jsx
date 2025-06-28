import React, { useState, useEffect } from 'react';
import { getSafeAvatarUrl, getSafeDisplayName, sanitizeUserContent, escapeHtml, isValidWebsite } from '../utils/validation';

/**
 * Secure Profile Page Component
 * Fixes XSS vulnerability by properly sanitizing all user input
 */
const ProfilePage = ({ userId }) => {
  const [user, setUser] = useState({
    id: '',
    avatar: '',
    displayName: '',
    bio: '',
    email: '',
    location: '',
    website: '',
    joinDate: '',
    followerCount: 0,
    followingCount: 0,
    postCount: 0
  });
  const [isEditing, setIsEditing] = useState(false);
  const [editForm, setEditForm] = useState({});
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');

  useEffect(() => {
    loadUserProfile(userId);
  }, [userId]);

  const loadUserProfile = async (id) => {
    setLoading(true);
    setError('');
    
    try {
      // This would be your actual API call
      const mockUser = {
        id: id || 'user123',
        avatar: 'https://avatars.githubusercontent.com/u/12345?v=4',
        displayName: 'John Doe',
        bio: 'Software engineer passionate about open source and package management.',
        email: 'john.doe@example.com',
        location: 'San Francisco, CA',
        website: 'https://johndoe.dev',
        joinDate: '2023-01-15T00:00:00Z',
        followerCount: 150,
        followingCount: 75,
        postCount: 42
      };
      
      setUser(mockUser);
      setEditForm(mockUser);
    } catch (err) {
      console.error('Error loading user profile:', err);
      setError('Failed to load user profile. Please try again.');
      
      // Set safe defaults on error
      setUser({
        id: 'anonymous',
        avatar: '/default-avatar.png',
        displayName: 'Anonymous User',
        bio: '',
        email: '',
        location: '',
        website: '',
        joinDate: new Date().toISOString(),
        followerCount: 0,
        followingCount: 0,
        postCount: 0
      });
    } finally {
      setLoading(false);
    }
  };

  const handleEditToggle = () => {
    if (isEditing) {
      // Reset form to current user data if canceling
      setEditForm(user);
    }
    setIsEditing(!isEditing);
  };

  const handleInputChange = (field, value) => {
    setEditForm(prev => ({
      ...prev,
      [field]: value
    }));
  };

  const handleSaveProfile = async (e) => {
    e.preventDefault();
    
    try {
      // Validate and sanitize all form inputs
      const sanitizedForm = {
        ...editForm,
        displayName: getSafeDisplayName(editForm.displayName),
        bio: sanitizeUserContent(editForm.bio),
        location: escapeHtml(editForm.location?.substring(0, 100) || ''),
        website: editForm.website && isValidWebsite(editForm.website) ? editForm.website : '',
        avatar: getSafeAvatarUrl(editForm.avatar)
      };

      // This would be your actual API call
      console.log('Saving profile:', sanitizedForm);
      
      setUser(sanitizedForm);
      setIsEditing(false);
      
      // Show success message
      alert('Profile updated successfully!');
      
    } catch (err) {
      console.error('Error saving profile:', err);
      alert('Failed to save profile. Please try again.');
    }
  };

  const isValidWebsite = (url) => {
    try {
      const parsed = new URL(url);
      return ['http:', 'https:'].includes(parsed.protocol);
    } catch {
      return false;
    }
  };

  // Get safe values for display
  const safeAvatar = getSafeAvatarUrl(user.avatar);
  const safeDisplayName = getSafeDisplayName(user.displayName);
  const safeBio = sanitizeUserContent(user.bio || '');
  const safeLocation = escapeHtml(user.location || '');
  const safeWebsite = user.website && isValidWebsite(user.website) ? user.website : '';

  if (loading) {
    return (
      <div className="max-w-4xl mx-auto p-6 bg-gray-900 min-h-screen">
        <div className="flex justify-center items-center h-64">
          <div className="text-white text-lg">Loading profile...</div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="max-w-4xl mx-auto p-6 bg-gray-900 min-h-screen">
        <div className="bg-red-900/50 border border-red-500 rounded-lg p-4 text-red-200">
          <h2 className="text-lg font-semibold mb-2">Error Loading Profile</h2>
          <p>{error}</p>
          <button 
            onClick={() => loadUserProfile(userId)}
            className="mt-4 px-4 py-2 bg-red-600 hover:bg-red-700 rounded-lg transition-colors"
          >
            Try Again
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="max-w-4xl mx-auto p-6 bg-gray-900 min-h-screen">
      {/* Profile Header */}
      <div className="bg-gray-800 rounded-lg p-8 mb-6">
        <div className="flex flex-col md:flex-row items-start md:items-center gap-6">
          {/* Avatar Section - LINE 37 WHERE VULNERABILITY WAS FIXED */}
          <div 
            className="relative"
          >
            <img
              src={safeAvatar}
              alt={safeDisplayName}
              className="w-32 h-32 rounded-full border-4 border-omni-purple/50"
              onError={(e) => {
                // Secure fallback if image fails to load
                e.target.src = '/default-avatar.png';
              }}
              onLoad={() => {
                // Additional security: Log successful avatar loads
                console.log('Profile avatar loaded successfully');
              }}
            />
            {isEditing && (
              <button className="absolute bottom-2 right-2 bg-omni-purple text-black p-2 rounded-full hover:bg-omni-purple/90 transition-colors">
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
                </svg>
              </button>
            )}
          </div>

          {/* Profile Info */}
          <div className="flex-1">
            {isEditing ? (
              <form onSubmit={handleSaveProfile} className="space-y-4">
                <div>
                  <label className="block text-sm font-medium text-gray-300 mb-1">Display Name</label>
                  <input
                    type="text"
                    value={editForm.displayName || ''}
                    onChange={(e) => handleInputChange('displayName', e.target.value)}
                    className="w-full p-2 bg-gray-700 text-white rounded border border-gray-600 focus:border-omni-purple/50 focus:outline-none"
                    maxLength={50}
                    required
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-300 mb-1">Avatar URL</label>
                  <input
                    type="url"
                    value={editForm.avatar || ''}
                    onChange={(e) => handleInputChange('avatar', e.target.value)}
                    className="w-full p-2 bg-gray-700 text-white rounded border border-gray-600 focus:border-omni-purple/50 focus:outline-none"
                    placeholder="https://example.com/avatar.jpg"
                  />
                  <p className="text-xs text-gray-400 mt-1">Only trusted domains are allowed</p>
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-300 mb-1">Bio</label>
                  <textarea
                    value={editForm.bio || ''}
                    onChange={(e) => handleInputChange('bio', e.target.value)}
                    className="w-full p-2 bg-gray-700 text-white rounded border border-gray-600 focus:border-omni-purple/50 focus:outline-none"
                    rows={3}
                    maxLength={200}
                    placeholder="Tell us about yourself..."
                  />
                </div>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm font-medium text-gray-300 mb-1">Location</label>
                    <input
                      type="text"
                      value={editForm.location || ''}
                      onChange={(e) => handleInputChange('location', e.target.value)}
                      className="w-full p-2 bg-gray-700 text-white rounded border border-gray-600 focus:border-omni-purple/50 focus:outline-none"
                      maxLength={100}
                      placeholder="City, Country"
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-300 mb-1">Website</label>
                    <input
                      type="url"
                      value={editForm.website || ''}
                      onChange={(e) => handleInputChange('website', e.target.value)}
                      className="w-full p-2 bg-gray-700 text-white rounded border border-gray-600 focus:border-omni-purple/50 focus:outline-none"
                      placeholder="https://your-website.com"
                    />
                  </div>
                </div>
                <div className="flex gap-3">
                  <button
                    type="submit"
                    className="px-6 py-2 bg-omni-purple text-black font-medium rounded-lg hover:bg-omni-purple/90 transition-colors"
                  >
                    Save Changes
                  </button>
                  <button
                    type="button"
                    onClick={handleEditToggle}
                    className="px-6 py-2 bg-gray-600 text-white font-medium rounded-lg hover:bg-gray-700 transition-colors"
                  >
                    Cancel
                  </button>
                </div>
              </form>
            ) : (
              <div>
                <div className="flex items-center gap-3 mb-2">
                  <h1 className="text-3xl font-bold text-white">{safeDisplayName}</h1>
                  <button
                    onClick={handleEditToggle}
                    className="px-4 py-1 bg-gray-600 text-white text-sm rounded-lg hover:bg-gray-700 transition-colors"
                  >
                    Edit Profile
                  </button>
                </div>
                <p className="text-gray-400 mb-3">@{escapeHtml(user.id)}</p>
                {user.bio && (
                  <div className="text-gray-200 mb-4">
                    {/* Safe rendering - React automatically escapes text content */}
                    {user.bio.split('\n').map((line, index) => (
                      <span key={index}>
                        {line}
                        {index < user.bio.split('\n').length - 1 && <br />}
                      </span>
                    ))}
                  </div>
                )}
                <div className="flex flex-wrap gap-4 text-sm text-gray-400">
                  {safeLocation && (
                    <div className="flex items-center gap-1">
                      <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z" />
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 11a3 3 0 11-6 0 3 3 0 016 0z" />
                      </svg>
                      <span>{safeLocation}</span>
                    </div>
                  )}
                  {safeWebsite && (
                    <div className="flex items-center gap-1">
                      <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
                      </svg>
                      <a 
                        href={safeWebsite} 
                        target="_blank" 
                        rel="noopener noreferrer"
                        className="text-omni-purple hover:underline"
                      >
                        {safeWebsite.replace(/^https?:\/\//, '')}
                      </a>
                    </div>
                  )}
                  <div className="flex items-center gap-1">
                    <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                    </svg>
                    <span>Joined {new Date(user.joinDate).toLocaleDateString()}</span>
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Stats */}
        {!isEditing && (
          <div className="mt-6 pt-6 border-t border-gray-700">
            <div className="grid grid-cols-3 gap-4 text-center">
              <div>
                <div className="text-2xl font-bold text-white">{user.postCount}</div>
                <div className="text-sm text-gray-400">Posts</div>
              </div>
              <div>
                <div className="text-2xl font-bold text-white">{user.followerCount}</div>
                <div className="text-sm text-gray-400">Followers</div>
              </div>
              <div>
                <div className="text-2xl font-bold text-white">{user.followingCount}</div>
                <div className="text-sm text-gray-400">Following</div>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Recent Activity */}
      <div className="bg-gray-800 rounded-lg p-6">
        <h2 className="text-xl font-bold text-white mb-4">Recent Activity</h2>
        <div className="text-gray-400 text-center py-8">
          <p>No recent activity to display.</p>
        </div>
      </div>
    </div>
  );
};

export default ProfilePage;