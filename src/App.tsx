import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { RiDeleteBin6Line } from "react-icons/ri";
import { RiFolder2Line } from "react-icons/ri";
import { open } from '@tauri-apps/plugin-shell';

const openInExplorer = (filePath: string) => {
  open(filePath)
    .then(() => console.log("Opened:", filePath))
    .catch((err) => console.error("Failed to open:", err));
};

type FileInfo = {
  name: string;
  path: string;
  size: number;
};

type FileGroup = {
  key: string;
  files: FileInfo[];
};

function App() {
  const [files, setFiles] = useState<FileInfo[]>([]);
  const [groups, setGroups] = useState<FileGroup[] | null>(null);
  const [view, setView] = useState<"all" | "extension" | "date">("all");

  useEffect(() => {
    invoke<FileInfo[]>("get_downloads_files")
      .then(setFiles)
      .catch(console.error);
  }, []);

  const loadGroupedByExtension = () => {
    invoke<FileGroup[]>("group_files_by_extension")
      .then((data) => {
        setGroups(data);
        setView("extension");
      })
      .catch(console.error);
  };

  const loadGroupedByDate = () => {
    invoke<FileGroup[]>("group_files_by_modified_date")
      .then((data) => {
        setGroups(data);
        setView("date");
      })
      .catch(console.error);
  };

  const loadAll = () => {
    invoke<FileInfo[]>("get_downloads_files")
      .then((data) => {
        setFiles(data);
        setGroups(null);
        setView("all");
      })
      .catch(console.error);
  };

  return (
    <div style={{ padding: "1rem" }}>
      <h1 className="text-orange-400 mb-4 text-2xl border-b-2 border-white text-left">DownSum</h1>
      <h2 className="mb-4 text-xl border-b-2 border-white text-left">Downloads Folder</h2>

      <div className="flex gap-4 mb-4">
        <button onClick={loadAll} className="bg-gray-700 px-3 py-1 rounded">
          All Files
        </button>
        <button onClick={loadGroupedByExtension} className="bg-blue-700 px-3 py-1 rounded">
          Group by Extension
        </button>
        <button onClick={loadGroupedByDate} className="bg-green-700 px-3 py-1 rounded">
          Group by Date
        </button>
        
      </div>

      {view === "all" && (
        <ul>
          {files.map((file) => (
            <li
              key={file.path}
              className="bg-orange-900 my-2 p-2 rounded-xl border-1 border-white"
            >
              
              {file.name} — {(file.size / 1024).toFixed(2)} KB
              <RiDeleteBin6Line></RiDeleteBin6Line>
            </li>
          ))}
        </ul>
      )}

      {(view === "extension" || view === "date") && groups && (
        <div className="space-y-4">
          {groups.map((group) => (
            <div key={group.key}>
              <h2 className="text-xl font-bold mb-2">{group.key}</h2>
              <ul>
                {group.files.map((file) => (
                 
                  <li
                    key={file.path}
                    className="my-3 p-2  border-1 "
                  >
                    
                    <div className="flex flex-row">
                    
                    {file.name} — {(file.size / 1024).toFixed(2)} KB 
                    <RiDeleteBin6Line className="hover:text-red-500 hover:cursor-pointer  w-5 h-5 rounded-full absolute right-8"></RiDeleteBin6Line>
                    <button type="button" onClick={() => openInExplorer(file.path)}>
                      open
                    </button>
                  </div>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

export default App;
