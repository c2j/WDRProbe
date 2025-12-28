#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

const filePath = path.join(__dirname, 'pages', 'PlanVisualizer.tsx');

try {
  const content = fs.readFileSync(filePath, 'utf8');
  const lines = content.split('\n');
  
  // Find the line with the valid function
  const validFunctionLine = lines.findIndex(line => 
    line.trim().startsWith('const handleSqlPlanExample = () => {') && 
    lines.indexOf(line) > 100 // Make sure it's not the first occurrence
  );
  
  if (validFunctionLine === -1) {
    console.error('Could not find valid function definition');
    process.exit(1);
  }
  
  // Find the end of the previous valid block (look for the closing brace)
  let endOfValidBlock = 56; // We know line 56 has "};"
  
  // Remove all lines between endOfValidBlock and validFunctionLine
  const cleanLines = lines.slice(0, endOfValidBlock + 1).concat(lines.slice(validFunctionLine));
  
  fs.writeFileSync(filePath, cleanLines.join('\n'), 'utf8');
  console.log(`✅ Fixed PlanVisualizer.tsx: Removed ${validFunctionLine - endOfValidBlock - 1} corrupted lines`);
  
} catch (error) {
  console.error('Error fixing file:', error);
  process.exit(1);
}