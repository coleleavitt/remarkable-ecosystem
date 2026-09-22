import { Cloud, Heart } from 'lucide-react';

export function About() {
  return (
    <div className="flex flex-col items-center justify-center h-full p-8 text-center">
      <Cloud className="w-24 h-24 text-gray-300 mb-6" />
      
      <h1 className="text-3xl font-bold text-gray-800 mb-2">reMarkable Desktop Sync</h1>
      <p className="text-gray-500 mb-6">Version 0.1.0</p>
      
      <div className="max-w-md text-gray-600 space-y-4">
        <p>
          A desktop application for syncing your reMarkable documents
          with the cloud or a local server.
        </p>
        
        <div className="flex items-center justify-center gap-1 text-sm">
          <span>Made with</span>
          <Heart className="w-4 h-4 text-red-500" />
          <span>using Tauri + React</span>
        </div>
      </div>
      
      <div className="mt-8 pt-8 border-t w-full max-w-md">
        <h2 className="font-semibold text-gray-700 mb-4">Features</h2>
        <ul className="text-left text-sm text-gray-600 space-y-2">
          <li>✓ Background sync with cloud or local server</li>
          <li>✓ System tray integration</li>
          <li>✓ Sync status notifications</li>
          <li>✓ Folder selection for partial sync</li>
          <li>✓ Offline mode support</li>
          <li>✓ Conflict resolution UI</li>
          <li>✓ Configurable sync interval</li>
        </ul>
      </div>
      
      <a
        href="https://github.com/coleleavitt/remarkable-desktop"
        target="_blank"
        rel="noopener noreferrer"
        className="mt-8 flex items-center gap-2 px-4 py-2 bg-gray-800 text-white rounded hover:bg-gray-700"
      >
        <span className="font-bold">GitHub</span>
        View on GitHub
      </a>
    </div>
  );
}

export default About;
