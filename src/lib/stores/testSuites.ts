import { writable, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { TestSuite, TestCase, TestRunResult } from '$lib/types';
import { newTestSuite, newTestCase } from '$lib/types';

export const testSuites = writable<TestSuite[]>([]);
export const testResults = writable<TestRunResult | null>(null);
export const runningTests = writable(false);

export function addTestSuite(name: string, collectionId: string) {
  const suite = newTestSuite(name, collectionId);
  testSuites.update(s => [...s, suite]);
  saveTestSuites();
}

export function deleteTestSuite(id: string) {
  testSuites.update(s => s.filter(suite => suite.id !== id));
  saveTestSuites();
}

export function addTestCase(suiteId: string) {
  const testCase = newTestCase();
  testSuites.update(suites =>
    suites.map(suite =>
      suite.id === suiteId
        ? { ...suite, test_cases: [...suite.test_cases, testCase] }
        : suite
    )
  );
  saveTestSuites();
}

export function updateTestCase(suiteId: string, testCase: TestCase) {
  testSuites.update(suites =>
    suites.map(suite =>
      suite.id === suiteId
        ? { ...suite, test_cases: suite.test_cases.map(tc => tc.id === testCase.id ? testCase : tc) }
        : suite
    )
  );
  saveTestSuites();
}

export function deleteTestCase(suiteId: string, testCaseId: string) {
  testSuites.update(suites =>
    suites.map(suite =>
      suite.id === suiteId
        ? { ...suite, test_cases: suite.test_cases.filter(tc => tc.id !== testCaseId) }
        : suite
    )
  );
  saveTestSuites();
}

export async function runTestSuite(suite: TestSuite, baseUrl: string) {
  runningTests.set(true);
  testResults.set(null);
  
  try {
    const result = await invoke<TestRunResult>('run_test_suite', { suite, baseUrl });
    testResults.set(result);
  } catch (e) {
    console.error('Test run failed:', e);
  } finally {
    runningTests.set(false);
  }
}

function saveTestSuites() {
  const current = get(testSuites);
  localStorage.setItem('tarasque-test-suites', JSON.stringify(current));
}

export function loadTestSuites() {
  const stored = localStorage.getItem('tarasque-test-suites');
  if (stored) {
    try {
      testSuites.set(JSON.parse(stored));
    } catch (e) {
      console.error('Failed to load test suites:', e);
    }
  }
}
