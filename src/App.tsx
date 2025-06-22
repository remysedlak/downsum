import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import FileIcons from "./comps/FileIcons";

import { TiMinus } from "react-icons/ti";

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

  const [expandedGroups, setExpandedGroups] = useState<Record<string, boolean>>({});

  const toggleGroup = (type: string) => {
  setExpandedGroups(prev => ({
    ...prev,
    [type]: !prev[type],
  }));
};

  useEffect(() => {
    invoke<FileInfo[]>("get_downloads_files")
      .then(setFiles)
      .catch(console.error);
  }, []);

useEffect(() => {
  if (!groups) return;

  const initialExpanded: Record<string, boolean> = {};
  for (const group of groups) {
    initialExpanded[group.key] = true;
  }
  setExpandedGroups(initialExpanded);
}, [groups]); // <-- depend on 'groups'

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
      {/*Title */}
      <h1 className="text-orange-400 mb-4 text-2xl border-b-2 border-white text-left">
        DownSum
      </h1>
      {/*Util Bar */}
      <div className="flex flex-row items-center gap-2 mb-4 border-b-2 border-white">
        <h2 className="mb-4 text-xl  text-left">Downloads Folder</h2>
        <div className="flex gap-4 mb-4">
          <button
            onClick={loadAll}
            className="bg-stone-800 px-3 py-1 rounded border-1 border-white"
          >
            All Files
          </button>
          <button
            onClick={loadGroupedByExtension}
            className=" px-3 py-1 rounded bg-stone-800 border-1 border-white"
          >
            Group by Extension
          </button>
          <button
            onClick={loadGroupedByDate}
            className="bg-stone-800 px-3 py-1 rounded border-1 border-white"
          >
            Group by Date
          </button>
          <button
            onClick={loadGroupedByDate}
            className="bg-green-700 px-3 py-1 rounded border-1 border-white"
          >
            Save Info
          </button>
        </div>
      </div>

      {/* File List */}
      {view === "all" && (
        <ul>
          <h2 className="text-xl font-bold mb-2">All files</h2>
          {files.map((file) => (
            <div>
              
              <li
                key={file.path}
                className="my-3 p-2 bg-stone-800 text-white border-1 relative"
              >
                <div className="flex flex-row">
                  {file.name} — {(file.size / 1024).toFixed(2)} KB
                  <FileIcons path={file.path} />
                </div>
              </li>
            </div>
          ))}
        </ul>
      )}
      {/* Grouped by Extension or Date */}
      {(view === "extension" || view === "date") && groups && (
        <div className="space-y-4">
          {groups.map((group) => (
            <div key={group.key}>
              <div className="flex flex-row items-center">
              <h2 className="text-xl font-bold mb-2 ">{group.key}</h2>
              <button className="ml-4 border-1 px-1" onClick={() => toggleGroup(group.key)}>
      {expandedGroups[group.key] ? 'Hide' : 'Show'} {/*{group.key}*/}
    </button>
              </div>
              <ul>
                {expandedGroups[group.key] && group.files.map((file) => (
                  <li key={file.path} className="my-3 p-2  border-1 ">
                    <div className="flex flex-row">
                      {file.name} — {(file.size / 1024).toFixed(2)} KB
                      <FileIcons path={file.path} />
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
