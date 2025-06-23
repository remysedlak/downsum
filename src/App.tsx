import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import FileIcons from "./comps/FileIcons";
import { openPath } from "@tauri-apps/plugin-opener";

type FileInfo = {
  name: string;
  path: string;
  size: number;
};

type FileGroup = {
  key: string;
  files: FileInfo[];
};

export type DuplicateFile = {
  name: string;
  path: string;
  size: number;
  duplicate_type: "exact" | "numbered" | "original" | "unknown";
};

export type DuplicateGroup = {
  original_name: string;
  files: DuplicateFile[];
  total_size: number;
};

function App() {
  const [files, setFiles] = useState<FileInfo[]>([]);
  const [groups, setGroups] = useState<FileGroup[] | null>(null);
  const [view, setView] = useState<"all" | "extension" | "date" | "duplicates">("all");
  const [duplicateGroups, setDuplicateGroups] = useState<DuplicateGroup[]>([]);
  const [expandedGroups, setExpandedGroups] = useState<Record<string, boolean>>({});
  const [showGroupList, setShowGroupList] = useState<boolean>(false);  // starts as false

  const toggleGroup = (key: string) => {
    setExpandedGroups(prev => ({
      ...prev,
      [key]: !prev[key],
    }));
  };

  // Load all files initially
  useEffect(() => {
    invoke<FileInfo[]>("get_downloads_files")
      .then(setFiles)
      .catch(console.error);
  }, []);

  const toggleShowGroupList = () => {
    setShowGroupList(prev => !prev);
  };

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

  const loadDuplicatesMode = async () => {
    try {
      const data = await invoke<DuplicateGroup[]>("find_duplicate_files");
      setDuplicateGroups(data);
      setView("duplicates");

      // Expand all groups by default
      const expanded: Record<string, boolean> = {};
      for (const group of data) {
        expanded[group.original_name] = true;
      }
      setExpandedGroups(expanded);
    } catch (error) {
      console.error("Failed to load duplicates:", error);
    }
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
      {/* Title */}
      <div className="flex flex-row justify-between border-b-2 border-white mb-4 pb-2 items-center">
        <p className="text-orange-400 text-2xl text-left">DownSum</p>
        <p className="text-md text-blue-400">built by remy</p>
      </div>

      {/* Util Bar */}
      <div className="flex flex-row items-center gap-2 mb-4 border-b-2 border-white ">
        <h2 className="mb-4 text-xl text-left">Downloads Folder</h2>
        <div className="ml-8 flex flex-row gap-4 mb-4 text-sm ml-auto ">
          {view !== "all" && (
          <button onClick={loadAll} className="bg-stone-800 px-3 py-1 rounded border-1 border-white">
            Clear filters
          </button>
          )}
          <div className="flex flex-col">
          <button onClick={() => toggleShowGroupList()} className="w-max px-3 py-1 rounded bg-stone-800 border-1 border-white">Group by</button>
          {showGroupList && (
            <div className="absolute w-full">
            <button onClick={loadGroupedByExtension} className="top-7 w-max absolute px-3 py-1 rounded bg-stone-800 border-1 border-white">
              Extension
            </button>
            <button onClick={loadGroupedByDate} className="top-14 absolute bg-stone-800 px-3 py-1 rounded border-1 border-white">
              Date
            </button>
          </div>
          
          )}
          </div>
          <button onClick={loadDuplicatesMode} className="bg-stone-800 px-3 py-1 rounded border-1 border-white">
            Find Duplicates
          </button>
        </div>
      </div>

      {/* Duplicates View */}
      {view === "duplicates" && (
        <div>
          <h2 className="text-xl font-bold mb-4">Duplicate Files</h2>
          {duplicateGroups.length === 0 && (
            <p className="text-gray-400">No duplicates found.</p>
          )}
          {duplicateGroups.map((group) => (
            <div key={group.original_name} className="mb-4 p-3 border rounded bg-stone-900">
              <div className="flex justify-between items-center">
                <h3 className="text-lg font-semibold">{group.original_name}</h3>
                <button
                  className="text-sm px-2 py-1 border rounded border-white"
                  onClick={() => toggleGroup(group.original_name)}
                >
                  {expandedGroups[group.original_name] ? "Hide" : "Show"}
                </button>
              </div>
              {expandedGroups[group.original_name] && (
                <ul className="mt-2 space-y-2">
                  {group.files.map((file) => (
                    <li key={file.path} className="p-2 bg-stone-800 flex justify-between rounded">
                      <div className="flex gap-4">
                        <span>{file.name}</span>
                        <span className="text-sm text-gray-400">
                          {file.duplicate_type} — {(file.size / 1024).toFixed(2)} KB
                        </span>
                      </div>
                      <FileIcons path={file.path} />
                    </li>
                  ))}
                </ul>
              )}
            </div>
          ))}
        </div>
      )}

      {/* All Files View */}
      {view === "all" && (
        <ul>
          <h2 className="text-xl font-bold mb-2">All files</h2>
          {files.map((file) => (
            <li
              key={file.path}
              className="my-3 p-2 bg-stone-800 text-white border-1 relative"
            >
              <div className="flex flex-row">
                <a className="hover:cursor-pointer mr-2 text-blue-300 hover:text-red-400" onClick={() => {
                                    console.log("Trying to open:", file.path);
                                    openPath(file.path)
                                        .then(() => console.log("Opened:", file.path))
                                        .catch((err ) =>
                                            console.error("Failed to open:", err)
                                        );
                                }}>{file.name}</a>
                 — {(file.size / 1024).toFixed(2)} KB
                <FileIcons path={file.path} />
              </div>
            </li>
          ))}
        </ul>
      )}

      {/* Grouped by Extension or Date */}
      {(view === "extension" || view === "date") && groups && (
        <div className="space-y-4">
          {groups.map((group) => (
            <div key={group.key}>
              <div className="flex flex-row items-center">
                <h2 className="text-xl font-bold mb-2">{group.key}</h2>
                <button className="ml-4 border-1 px-1" onClick={() => toggleGroup(group.key)}>
                  {expandedGroups[group.key] ? "Hide" : "Show"}
                </button>
              </div>
              <ul>
                {expandedGroups[group.key] &&
                  group.files.map((file) => (
                    <li key={file.path} className="my-3 p-2 border-1">
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
