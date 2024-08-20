import { IPC } from './index';
import type { Ipc as GenericIpc } from './index';

type IpcIncomingMessage =
  | {
      type: 'evaluate';
      args: string[];
    }
  | {
      type: 'result';
      id: number;
      error: string | null;
      data: any | null;
    };

type IpcOutgoingMessage =
  | {
      type: 'end';
      data: string | undefined;
      duration: number;
    }
  | {
      type: 'info';
      data: any;
    }
  | {
      type: 'request';
      id: number;
      data: any;
    };

export type Ipc<IM, RM> = {
  sendInfo(message: IM): Promise<void>;
  sendRequest(message: RM): Promise<unknown>;
  sendError(error: Error): Promise<never>;
};
const ipc = IPC as GenericIpc<IpcIncomingMessage, IpcOutgoingMessage>;

const queue: string[][] = [];

export const run = async (
  moduleFactory: () => Promise<{
    init?: () => Promise<void>;
    default: (...deserializedArgs: any[]) => any;
  }>,
) => {
  const requests = new Map();

  // Initialize module and send ready message
  let getValue: (...deserializedArgs: any[]) => any;
  try {
    const module = await moduleFactory();
    if (typeof module.init === 'function') {
      await module.init();
    }
    getValue = module.default;
    await ipc.sendReady();
  } catch (err) {
    await ipc.sendReady();
    await ipc.sendError(err as Error);
  }

  // Queue handling
  let isRunning = false;
  const run = async () => {
    while (queue.length > 0) {
      const args = queue.shift()!;
      try {
        const value = await getValue(...args);
        await ipc.send({
          type: 'end',
          data:
            value === undefined ? undefined : JSON.stringify(value, null, 2),
          duration: 0,
        });
      } catch (e) {
        await ipc.sendError(e as Error);
      }
    }
    isRunning = false;
  };

  // Communication handling
  while (true) {
    const msg = await ipc.recv();

    switch (msg.type) {
      case 'evaluate': {
        queue.push(msg.args);
        if (!isRunning) {
          isRunning = true;
          run();
        }
        break;
      }
      case 'result': {
        const request = requests.get(msg.id);
        if (request) {
          requests.delete(msg.id);
          if (msg.error) {
            request.reject(new Error(msg.error));
          } else {
            request.resolve(msg.data);
          }
        }
        break;
      }
      default: {
        console.error('unexpected message type', (msg as any).type);
        process.exit(1);
      }
    }
  }
};

export const getExportedValues = async <T extends string>(
  exportNames: T[],
  moduleFactory: () => Promise<
    {
      init?: () => Promise<void>;
    } & {
      [K in T]: any;
    }
  >,
) => {
  const requests = new Map();

  // Initialize module and send ready message
  let getValue: () => any;
  try {
    const module = await moduleFactory();
    if (typeof module.init === 'function') {
      await module.init();
    }
    getValue = async () =>
      Object.fromEntries(
        exportNames.map((exportName) => [exportName, module[exportName]]),
      );
    await ipc.sendReady();
  } catch (err) {
    await ipc.sendReady();
    await ipc.sendError(err as Error);
  }

  // Queue handling
  let isRunning = false;
  const run = async () => {
    while (queue.length > 0) {
      try {
        const value = await getValue();
        await ipc.send({
          type: 'end',
          data:
            value === undefined ? undefined : JSON.stringify(value, null, 2),
          duration: 0,
        });
      } catch (e) {
        await ipc.sendError(e as Error);
      }
    }
    isRunning = false;
  };

  // Communication handling
  while (true) {
    const msg = await ipc.recv();

    switch (msg.type) {
      case 'evaluate': {
        queue.push(msg.args);
        if (!isRunning) {
          isRunning = true;
          run();
        }
        break;
      }
      case 'result': {
        const request = requests.get(msg.id);
        if (request) {
          requests.delete(msg.id);
          if (msg.error) {
            request.reject(new Error(msg.error));
          } else {
            request.resolve(msg.data);
          }
        }
        break;
      }
      default: {
        console.error('unexpected message type', (msg as any).type);
        process.exit(1);
      }
    }
  }
};

export const getDefaultValue = async (
  moduleFactory: () => Promise<{
    init?: () => Promise<void>;
    default: any;
  }>,
) => {
  const requests = new Map();

  // Initialize module and send ready message
  let getValue: any;
  try {
    const module = await moduleFactory();
    if (typeof module.init === 'function') {
      await module.init();
    }
    getValue = module.default;
    await ipc.sendReady();
  } catch (err) {
    await ipc.sendReady();
    await ipc.sendError(err as Error);
  }

  // Queue handling
  let isRunning = false;
  const run = async () => {
    while (queue.length > 0) {
      const args = queue.shift()!;
      try {
        const value = await getValue;
        await ipc.send({
          type: 'end',
          data:
            value === undefined ? undefined : JSON.stringify(value, null, 2),
          duration: 0,
        });
      } catch (e) {
        await ipc.sendError(e as Error);
      }
    }
    isRunning = false;
  };

  // Communication handling
  while (true) {
    const msg = await ipc.recv();

    switch (msg.type) {
      case 'evaluate': {
        queue.push(msg.args);
        if (!isRunning) {
          isRunning = true;
          run();
        }
        break;
      }
      case 'result': {
        const request = requests.get(msg.id);
        if (request) {
          requests.delete(msg.id);
          if (msg.error) {
            request.reject(new Error(msg.error));
          } else {
            request.resolve(msg.data);
          }
        }
        break;
      }
      default: {
        console.error('unexpected message type', (msg as any).type);
        process.exit(1);
      }
    }
  }
};

export type { IpcIncomingMessage, IpcOutgoingMessage };
