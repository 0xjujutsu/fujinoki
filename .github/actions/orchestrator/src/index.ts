// @ts-expect-error
import { run } from '@probot/adapter-github-actions';
import app from './app';

run(app).catch((error) => {
  console.error(error);
  process.exit(1);
});
