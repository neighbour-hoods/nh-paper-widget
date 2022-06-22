import {CategoryProvider, Category} from "typescript-logging-category-style";

const provider = CategoryProvider.createProvider("ExampleProvider");

export function getLogger(name: string): Category {
  return provider.getCategory(name);
}