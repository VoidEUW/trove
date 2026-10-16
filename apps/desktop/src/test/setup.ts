// Runs before every test file: Testing Library's matchers for the DOM.
import '@testing-library/jest-dom/vitest';

// jsdom has no modal <dialog> and no canvas; these stand in for what the tests touch.
HTMLDialogElement.prototype.showModal ??= function (this: HTMLDialogElement) {
	this.setAttribute('open', '');
};
HTMLDialogElement.prototype.close ??= function (this: HTMLDialogElement) {
	this.removeAttribute('open');
	this.dispatchEvent(new Event('close'));
};
HTMLCanvasElement.prototype.getContext = () => null;
