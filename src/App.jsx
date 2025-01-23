import { useState, useEffect } from 'react';
import './App.css';

const sampleData = [
 { id: 1, type: 'app', name: 'Safari', icon: '🌐' },
 { id: 2, type: 'app', name: 'Settings', icon: '⚙️' },
 { id: 3, type: 'app', name: 'Music', icon: '🎵' },
 { id: 4, type: 'file', name: 'Document.pdf', icon: '📄' },
 { id: 5, type: 'file', name: 'Screenshot.png', icon: '🖼️' },
 { id: 5, type: 'file', name: 'Screenshot.png', icon: '🖼️' },
 { id: 5, type: 'file', name: 'Screenshot.png', icon: '🖼️' },
 { id: 5, type: 'file', name: 'Screenshot.png', icon: '🖼️' },
 { id: 5, type: 'file', name: 'Screenshot.png', icon: '🖼️' },
 { id: 5, type: 'file', name: 'Screenshot.png', icon: '🖼️' },
 { id: 5, type: 'file', name: 'Screenshot.png', icon: '🖼️' },
 { id: 5, type: 'file', name: 'Screenshot.png', icon: '🖼️' },
 { id: 5, type: 'file', name: 'Screenshot.png', icon: '🖼️' },
 { id: 6, type: 'command', name: 'Terminal', icon: '⌘' }
];

export default function App() {
 const [query, setQuery] = useState('');
 const [results, setResults] = useState(sampleData);
 const [selectedIndex, setSelectedIndex] = useState(0);

 useEffect(() => {
   const filtered = sampleData.filter(item =>
     item.name.toLowerCase().includes(query.toLowerCase())
   );
   setResults(filtered);
   setSelectedIndex(0);
 }, [query]);

 const handleKeyDown = (e) => {
   if (e.key === 'ArrowDown') {
     e.preventDefault();
     setSelectedIndex(prev => Math.min(prev + 1, results.length - 1));
   } else if (e.key === 'ArrowUp') {
     e.preventDefault();
     setSelectedIndex(prev => Math.max(prev - 1, 0));
   }
 };

 return (
   <div className="container">
     <div className="search">
       <input
         type="text"
         value={query}
         onChange={(e) => setQuery(e.target.value)}
         onKeyDown={handleKeyDown}
         placeholder="Surge Search"
         autoFocus
       />
     </div>
     
     <div className="results">
       {results.map((item, index) => (
         <div key={item.id} className={`result ${index === selectedIndex ? 'selected' : ''}`}>
           <span className="icon">{item.icon}</span>
           <div className="details">
             <span className="name">{item.name}</span>
             <span className="type">{item.type}</span>
           </div>
         </div>
       ))}
     </div>
   </div>
 );
}