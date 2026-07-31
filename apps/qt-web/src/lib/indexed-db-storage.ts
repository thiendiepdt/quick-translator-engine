import type { StateStorage } from "zustand/middleware";

const databaseName = "qt-web";
const databaseVersion = 1;
const objectStoreName = "key-value";

let databasePromise: Promise<IDBDatabase> | undefined;

function openDatabase(): Promise<IDBDatabase> {
  if (databasePromise) return databasePromise;

  const opening = new Promise<IDBDatabase>((resolve, reject) => {
    const request = window.indexedDB.open(databaseName, databaseVersion);
    request.onupgradeneeded = () => {
      const database = request.result;
      if (!database.objectStoreNames.contains(objectStoreName)) {
        database.createObjectStore(objectStoreName);
      }
    };
    request.onsuccess = () => {
      const database = request.result;
      database.onversionchange = () => {
        database.close();
        databasePromise = undefined;
      };
      resolve(database);
    };
    request.onerror = () => reject(request.error ?? new Error("Không mở được IndexedDB"));
  });
  const result = opening.catch((error: unknown) => {
    databasePromise = undefined;
    throw error;
  });
  databasePromise = result;
  return result;
}

async function readValue(key: string): Promise<string | null> {
  const database = await openDatabase();
  return new Promise((resolve, reject) => {
    const transaction = database.transaction(objectStoreName, "readonly");
    const request = transaction.objectStore(objectStoreName).get(key);
    request.onsuccess = () => {
      resolve(typeof request.result === "string" ? request.result : null);
    };
    request.onerror = () => reject(request.error ?? new Error("Không đọc được IndexedDB"));
  });
}

async function writeValue(key: string, value: string): Promise<void> {
  const database = await openDatabase();
  return new Promise((resolve, reject) => {
    const transaction = database.transaction(objectStoreName, "readwrite");
    transaction.objectStore(objectStoreName).put(value, key);
    transaction.oncomplete = () => resolve();
    transaction.onerror = () => reject(transaction.error ?? new Error("Không ghi được IndexedDB"));
    transaction.onabort = () => reject(transaction.error ?? new Error("Ghi IndexedDB bị hủy"));
  });
}

async function deleteValue(key: string): Promise<void> {
  const database = await openDatabase();
  return new Promise((resolve, reject) => {
    const transaction = database.transaction(objectStoreName, "readwrite");
    transaction.objectStore(objectStoreName).delete(key);
    transaction.oncomplete = () => resolve();
    transaction.onerror = () => reject(transaction.error ?? new Error("Không xóa được IndexedDB"));
    transaction.onabort = () => reject(transaction.error ?? new Error("Xóa IndexedDB bị hủy"));
  });
}

function localStorageValue(key: string): string | null {
  try {
    return window.localStorage.getItem(key);
  } catch {
    return null;
  }
}

function removeLocalStorageValue(key: string): void {
  try {
    window.localStorage.removeItem(key);
  } catch {
    // Không xóa được bản cũ không ảnh hưởng bản đã migrate sang IndexedDB.
  }
}

interface IndexedDbStateStorageOptions {
  legacyLocalStorageKeys?: readonly string[];
}

export function createIndexedDbStateStorage({
  legacyLocalStorageKeys = [],
}: IndexedDbStateStorageOptions = {}): StateStorage {
  let pendingWrite: Promise<void> = Promise.resolve();

  function enqueueWrite(operation: () => Promise<void>): Promise<void> {
    const result = pendingWrite.then(operation);
    pendingWrite = result.catch(() => undefined);
    return result;
  }

  return {
    async getItem(key) {
      await pendingWrite;
      const stored = await readValue(key);
      if (stored !== null) return stored;

      for (const legacyKey of legacyLocalStorageKeys) {
        const legacyValue = localStorageValue(legacyKey);
        if (legacyValue === null) continue;
        await writeValue(key, legacyValue);
        removeLocalStorageValue(legacyKey);
        return legacyValue;
      }
      return null;
    },
    setItem: (key, value) => enqueueWrite(() => writeValue(key, value)),
    removeItem: (key) => enqueueWrite(() => deleteValue(key)),
  };
}
