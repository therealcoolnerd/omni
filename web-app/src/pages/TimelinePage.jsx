import React, { useState, useEffect } from 'react';
import { getSafeAvatarUrl, getSafeDisplayName, sanitizeUserContent } from '../utils/validation';

/**
 * Secure Timeline Page Component
 * Fixes XSS vulnerability by properly sanitizing user input
 */
const TimelinePage = () => {
  const [posts, setPosts] = useState([]);
  const [newPost, setNewPost] = useState('');
  const [user, setUser] = useState({
    avatar: '',
    displayName: '',
    id: ''
  });

  useEffect(() => {
    // Load user data (this would come from your auth system)
    loadUserData();
    loadTimelinePosts();
  }, []);

  const loadUserData = async () => {
    try {
      // This would be your actual user data loading logic
      const userData = {
        avatar: 'https://avatars.githubusercontent.com/u/12345?v=4',
        displayName: 'John Doe',
        id: 'user123'
      };
      setUser(userData);
    } catch (error) {
      console.error('Error loading user data:', error);
      setUser({
        avatar: '/default-avatar.png',
        displayName: 'Anonymous User',
        id: 'anonymous'
      });
    }
  };

  const loadTimelinePosts = async () => {
    try {
      // This would be your actual posts loading logic
      const mockPosts = [
        {
          id: '1',
          content: 'Welcome to the secure timeline!',
          author: {
            avatar: 'https://avatars.githubusercontent.com/u/54321?v=4',
            displayName: 'Jane Smith'
          },
          timestamp: new Date().toISOString()
        }
      ];
      setPosts(mockPosts);
    } catch (error) {
      console.error('Error loading posts:', error);
      setPosts([]);
    }
  };

  const handleCreatePost = async (e) => {
    e.preventDefault();
    
    if (!newPost.trim()) {
      alert('Post content cannot be empty');
      return;
    }

    try {
      // Sanitize post content before sending to server
      const sanitizedContent = sanitizeUserContent(newPost);
      
      const postData = {
        content: sanitizedContent,
        author: {
          avatar: getSafeAvatarUrl(user.avatar),
          displayName: getSafeDisplayName(user.displayName)
        },
        timestamp: new Date().toISOString()
      };

      // This would be your actual post creation API call
      console.log('Creating post:', postData);
      
      // Add to local state (in real app, this would come from server response)
      setPosts(prevPosts => [
        { ...postData, id: Date.now().toString() },
        ...prevPosts
      ]);
      
      setNewPost('');
    } catch (error) {
      console.error('Error creating post:', error);
      alert('Failed to create post. Please try again.');
    }
  };

  // Get safe user data for display
  const safeAvatar = getSafeAvatarUrl(user.avatar);
  const safeDisplayName = getSafeDisplayName(user.displayName);

  return (
    <div className="max-w-2xl mx-auto p-6 bg-gray-900 min-h-screen">
      <header className="mb-8">
        <h1 className="text-3xl font-bold text-white mb-2">Timeline</h1>
        <p className="text-gray-400">Share your thoughts securely</p>
      </header>

      {/* Secure Post Creation Form - Line 162 where the vulnerability was */}
      <div className="bg-gray-800 rounded-lg p-6 mb-6">
        <form onSubmit={handleCreatePost} className="space-y-4">
          <div className="flex items-start space-x-3">
            <img
              src={safeAvatar}
              alt={safeDisplayName}
              className="w-12 h-12 rounded-full border-2 border-nukie-gold/30"
              onError={(e) => {
                // Fallback if image fails to load
                e.target.src = '/default-avatar.png';
              }}
              onLoad={(e) => {
                // Additional security: verify image loaded successfully
                console.log('Avatar loaded successfully');
              }}
            />
            <div className="flex-1">
              <div className="flex items-center mb-2">
                <span className="font-medium text-white">
                  {safeDisplayName}
                </span>
                <span className="text-gray-400 text-sm ml-2">
                  @{user.id || 'anonymous'}
                </span>
              </div>
              <textarea
                value={newPost}
                onChange={(e) => setNewPost(e.target.value)}
                placeholder="What's on your mind?"
                className="w-full p-3 bg-gray-700 text-white rounded-lg border border-gray-600 focus:border-nukie-gold/50 focus:outline-none resize-none"
                rows={3}
                maxLength={280}
                required
              />
              <div className="flex justify-between items-center mt-2">
                <span className="text-sm text-gray-400">
                  {280 - newPost.length} characters remaining
                </span>
                <button
                  type="submit"
                  disabled={!newPost.trim()}
                  className="px-6 py-2 bg-nukie-gold text-black font-medium rounded-lg hover:bg-nukie-gold/90 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                >
                  Post
                </button>
              </div>
            </div>
          </div>
        </form>
      </div>

      {/* Timeline Posts */}
      <div className="space-y-4">
        {posts.map((post) => (
          <TimelinePost key={post.id} post={post} />
        ))}
      </div>

      {posts.length === 0 && (
        <div className="text-center py-12">
          <p className="text-gray-400">No posts yet. Be the first to share something!</p>
        </div>
      )}
    </div>
  );
};

/**
 * Secure Timeline Post Component
 */
const TimelinePost = ({ post }) => {
  const safeAvatar = getSafeAvatarUrl(post.author.avatar);
  const safeDisplayName = getSafeDisplayName(post.author.displayName);
  const safeContent = sanitizeUserContent(post.content);

  return (
    <div className="bg-gray-800 rounded-lg p-6">
      <div className="flex items-start space-x-3">
        <img
          src={safeAvatar}
          alt={safeDisplayName}
          className="w-10 h-10 rounded-full border-2 border-gray-600"
          onError={(e) => {
            e.target.src = '/default-avatar.png';
          }}
        />
        <div className="flex-1">
          <div className="flex items-center mb-2">
            <span className="font-medium text-white">
              {safeDisplayName}
            </span>
            <span className="text-gray-400 text-sm ml-2">
              {new Date(post.timestamp).toLocaleDateString()}
            </span>
          </div>
          <div className="text-gray-200">
            {/* Safe content rendering - no dangerouslySetInnerHTML */}
            {safeContent}
          </div>
        </div>
      </div>
    </div>
  );
};

export default TimelinePage;