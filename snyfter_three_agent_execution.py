"""
Three-Agent SNYFTER Development Execution
Autonomous implementation using Pydantic, Instructor, and OpenHands
"""

import os
import sys
import time
import subprocess
from pathlib import Path
from datetime import datetime
from typing import Dict, List, Optional, Any
import json

# Import our orchestration blueprint
from snyfter_autonomous_orchestration import (
    SNYFTERBlueprint, DevelopmentTask, TaskType, TaskStatus,
    DatabaseTask, UIComponentTask, Priority, 
    start_caffeinate, stop_caffeinate
)

# Import our existing tools
from elegant_models import *
from development_tools_integration import DevelopmentWorkflow


class PydanticAgent:
    """Pydantic agent for data modeling and validation"""
    
    def __init__(self):
        self.workflow = DevelopmentWorkflow()
        
    def create_snyfter_models(self) -> str:
        """Create Pydantic models for SNYFTER features"""
        models_code = '''
"""
SNYFTER Data Models
Pydantic models for qualitative analysis features
"""

from pydantic import BaseModel, Field, validator
from typing import List, Optional, Dict
from datetime import datetime
from enum import Enum
import uuid


class CodeColor(str, Enum):
    """Predefined code colors"""
    TURQUOISE = "#1ABC9C"
    EMERALD = "#2ECC71"
    BLUE = "#3498DB"
    PURPLE = "#9B59B6"
    YELLOW = "#F1C40F"
    ORANGE = "#E67E22"
    RED = "#E74C3C"
    GRAY = "#95A5A6"


class QualitativeCode(BaseModel):
    """Model for qualitative analysis codes"""
    code_id: str = Field(default_factory=lambda: str(uuid.uuid4()))
    name: str = Field(..., min_length=1, max_length=100)
    description: Optional[str] = Field(None, max_length=500)
    color: CodeColor = Field(default=CodeColor.TURQUOISE)
    parent_id: Optional[str] = Field(None)
    created_date: datetime = Field(default_factory=datetime.now)
    modified_date: Optional[datetime] = None
    
    @validator('name')
    def validate_name(cls, v):
        if not v.strip():
            raise ValueError('Code name cannot be empty')
        return v.strip()


class Annotation(BaseModel):
    """Model for text annotations"""
    annotation_id: str = Field(default_factory=lambda: str(uuid.uuid4()))
    document_id: str = Field(...)
    code_id: str = Field(...)
    start_pos: int = Field(..., ge=0)
    end_pos: int = Field(..., gt=0)
    highlighted_text: str = Field(...)
    memo: Optional[str] = Field(None, max_length=2000)
    created_date: datetime = Field(default_factory=datetime.now)
    created_by: str = Field(default="user")
    
    @validator('end_pos')
    def validate_positions(cls, v, values):
        if 'start_pos' in values and v <= values['start_pos']:
            raise ValueError('End position must be greater than start position')
        return v
    
    @validator('highlighted_text')
    def validate_highlighted_text(cls, v):
        if not v.strip():
            raise ValueError('Highlighted text cannot be empty')
        return v


class CodeTree(BaseModel):
    """Model for hierarchical code structure"""
    root_codes: List[QualitativeCode] = Field(default_factory=list)
    child_map: Dict[str, List[QualitativeCode]] = Field(default_factory=dict)
    
    def add_code(self, code: QualitativeCode):
        """Add a code to the tree"""
        if code.parent_id is None:
            self.root_codes.append(code)
        else:
            if code.parent_id not in self.child_map:
                self.child_map[code.parent_id] = []
            self.child_map[code.parent_id].append(code)
    
    def get_all_codes(self) -> List[QualitativeCode]:
        """Get flat list of all codes"""
        all_codes = list(self.root_codes)
        for children in self.child_map.values():
            all_codes.extend(children)
        return all_codes
    
    def get_code_by_id(self, code_id: str) -> Optional[QualitativeCode]:
        """Find a code by ID"""
        for code in self.get_all_codes():
            if code.code_id == code_id:
                return code
        return None


class AnnotatedDocument(BaseModel):
    """Model for document with annotations"""
    document_id: str = Field(...)
    document_path: str = Field(...)
    title: str = Field(...)
    annotations: List[Annotation] = Field(default_factory=list)
    last_modified: datetime = Field(default_factory=datetime.now)
    
    def add_annotation(self, annotation: Annotation):
        """Add annotation ensuring no overlaps"""
        # Check for overlaps
        for existing in self.annotations:
            if (annotation.start_pos < existing.end_pos and 
                annotation.end_pos > existing.start_pos):
                raise ValueError(f"Annotation overlaps with existing annotation {existing.annotation_id}")
        
        self.annotations.append(annotation)
        self.last_modified = datetime.now()
    
    def get_annotations_by_code(self, code_id: str) -> List[Annotation]:
        """Get all annotations for a specific code"""
        return [a for a in self.annotations if a.code_id == code_id]
    
    def remove_annotation(self, annotation_id: str) -> bool:
        """Remove an annotation by ID"""
        for i, ann in enumerate(self.annotations):
            if ann.annotation_id == annotation_id:
                self.annotations.pop(i)
                self.last_modified = datetime.now()
                return True
        return False


class SNYFTERProject(BaseModel):
    """Model for complete SNYFTER project"""
    project_id: str = Field(default_factory=lambda: str(uuid.uuid4()))
    name: str = Field(..., min_length=1, max_length=200)
    description: Optional[str] = Field(None)
    code_tree: CodeTree = Field(default_factory=CodeTree)
    documents: Dict[str, AnnotatedDocument] = Field(default_factory=dict)
    created_date: datetime = Field(default_factory=datetime.now)
    last_saved: Optional[datetime] = None
    
    def add_document(self, doc: AnnotatedDocument):
        """Add a document to the project"""
        self.documents[doc.document_id] = doc
        
    def get_all_annotations(self) -> List[Annotation]:
        """Get all annotations across all documents"""
        annotations = []
        for doc in self.documents.values():
            annotations.extend(doc.annotations)
        return annotations
    
    def export_codebook(self) -> Dict[str, Any]:
        """Export code structure as dictionary"""
        return {
            "project_name": self.name,
            "codes": [code.dict() for code in self.code_tree.get_all_codes()],
            "total_annotations": len(self.get_all_annotations()),
            "export_date": datetime.now().isoformat()
        }
'''
        
        # Save the models
        models_path = Path("snyfter_models.py")
        models_path.write_text(models_code)
        
        return f"Created SNYFTER Pydantic models at {models_path}"
    
    def create_database_schema(self, task: DatabaseTask) -> str:
        """Generate database schema from task specification"""
        schema_sql = f"CREATE TABLE IF NOT EXISTS {task.table_name} (\n"
        
        # Add columns
        for col_name, col_type in task.schema.items():
            schema_sql += f"    {col_name} {col_type},\n"
        
        # Add relationships
        for relationship in task.relationships:
            schema_sql += f"    {relationship},\n"
        
        schema_sql = schema_sql.rstrip(",\n") + "\n);\n\n"
        
        # Add indexes
        for index in task.indexes:
            schema_sql += f"{index};\n"
        
        return schema_sql


class InstructorAgent:
    """Instructor agent for code generation and analysis"""
    
    def __init__(self):
        self.templates = {
            TaskType.DATABASE: self._database_template,
            TaskType.UI_COMPONENT: self._ui_component_template,
            TaskType.INTEGRATION: self._integration_template
        }
    
    def _database_template(self, task: DevelopmentTask) -> str:
        """Generate database extension code"""
        return f'''
def extend_database_for_snyfter(self):
    """Extend DocumentDatabase with SNYFTER tables"""
    
    # Add new tables for qualitative coding
    snyfter_schema = """
    CREATE TABLE IF NOT EXISTS codes (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        description TEXT,
        color TEXT DEFAULT '#1ABC9C',
        parent_id INTEGER,
        created_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (parent_id) REFERENCES codes(id)
    );
    
    CREATE TABLE IF NOT EXISTS annotations (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        document_id TEXT NOT NULL,
        start_pos INTEGER NOT NULL,
        end_pos INTEGER NOT NULL,
        highlighted_text TEXT,
        code_id INTEGER,
        memo TEXT,
        created_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (document_id) REFERENCES documents(id),
        FOREIGN KEY (code_id) REFERENCES codes(id)
    );
    
    CREATE INDEX idx_codes_parent ON codes(parent_id);
    CREATE INDEX idx_annotations_doc ON annotations(document_id);
    CREATE INDEX idx_annotations_code ON annotations(code_id);
    """
    
    with self.get_connection() as conn:
        conn.executescript(snyfter_schema)
        conn.commit()
'''
    
    def _ui_component_template(self, task: DevelopmentTask) -> str:
        """Generate UI component code"""
        # For UI component tasks, extract component details from task name/description
        component_name = task.name.replace("Create ", "").replace(" component", "").replace(" ", "")
        parent_class = "QWidget"  # Default parent class
        
        return f'''
class {component_name}({parent_class}):
    """Generated {component_name} for SNYFTER mode"""
    
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setup_ui()
        self.setup_style()
        self.setup_event_handlers()
    
    def setup_ui(self):
        """Initialize UI components"""
        # Generated UI setup
        pass
    
    def setup_style(self):
        """Apply SNYFTER theme styling"""
        style = f"""
        {component_name} {{
            background-color: #525659;
            color: #FFFFFF;
            border: 1px solid #3A3C3E;
        }}
        """
        self.setStyleSheet(style)
    
    def setup_event_handlers(self):
        """Connect event handlers"""
        # Generated event handler connections
        pass
    
    # Additional methods would be generated here based on component type
    def refresh(self):
        """Refresh component state"""
        pass
'''
    
    def _integration_template(self, task: DevelopmentTask) -> str:
        """Generate integration code"""
        return f'''
def integrate_snyfter_mode(self):
    """Integrate SNYFTER mode into existing application"""
    
    # Modify set_mode to handle SNYFTER
    if mode == Mode.SNYFTER:
        self.log("SNYFTER mode activated - Ready for qualitative analysis!")
        
        # Clear existing panes
        self._clear_panes()
        
        # Create SNYFTER layout
        self._create_snyfter_layout()
        
        # Preserve shared components
        self._preserve_shared_ui()
'''
    
    def _generate_methods(self, methods: List[str]) -> str:
        """Generate method stubs"""
        method_code = ""
        for method in methods:
            method_code += f'''
    def {method}(self):
        """Generated method: {method}"""
        # TODO: Implement {method}
        pass
'''
        return method_code
    
    def generate_code(self, task: DevelopmentTask) -> str:
        """Generate code for a task"""
        template = self.templates.get(task.type)
        if template:
            return template(task)
        return "# No template available for this task type"


class OpenHandsAgent:
    """OpenHands agent for task execution and integration"""
    
    def __init__(self):
        self.created_files = []
        self.modified_files = []
        self.backup_dir = Path("backups") / datetime.now().strftime("%Y%m%d_%H%M%S")
        self.backup_dir.mkdir(parents=True, exist_ok=True)
    
    def backup_file(self, filepath: str):
        """Create backup of existing file"""
        source = Path(filepath)
        if source.exists():
            backup_path = self.backup_dir / source.name
            backup_path.write_text(source.read_text())
            return backup_path
        return None
    
    def execute_task(self, task: DevelopmentTask, code: str) -> bool:
        """Execute a development task"""
        try:
            for file_spec in task.files_to_modify:
                if ':' in file_spec:
                    filename, target = file_spec.split(':', 1)
                else:
                    filename = file_spec
                    target = None
                
                if filename == "new":
                    # Create new file
                    new_file = Path(target)
                    new_file.write_text(code)
                    self.created_files.append(str(new_file))
                else:
                    # Modify existing file
                    filepath = Path(filename)
                    if filepath.exists():
                        # Backup first
                        self.backup_file(str(filepath))
                        
                        if target:
                            # Insert code at specific location
                            self._insert_code_at_target(filepath, target, code)
                        else:
                            # Append code
                            existing = filepath.read_text()
                            filepath.write_text(existing + "\n\n" + code)
                        
                        self.modified_files.append(str(filepath))
                    else:
                        # Create new file
                        filepath.write_text(code)
                        self.created_files.append(str(filepath))
            
            return True
            
        except Exception as e:
            print(f"Error executing task: {e}")
            return False
    
    def _insert_code_at_target(self, filepath: Path, target: str, code: str):
        """Insert code at specific target location"""
        content = filepath.read_text()
        
        # Simple insertion at class or method level
        if "." in target:
            class_name, method_name = target.split(".", 1)
            # Find class definition
            class_pattern = f"class {class_name}"
            if class_pattern in content:
                # Find insertion point
                lines = content.split('\n')
                for i, line in enumerate(lines):
                    if class_pattern in line:
                        # Find end of class or next method
                        indent = len(line) - len(line.lstrip())
                        for j in range(i+1, len(lines)):
                            if lines[j].strip() and not lines[j].startswith(' ' * (indent + 4)):
                                # Insert before this line
                                lines.insert(j, code)
                                break
                        else:
                            # Append to end
                            lines.append(code)
                        break
                
                filepath.write_text('\n'.join(lines))
        else:
            # Just append for now
            filepath.write_text(content + "\n\n" + code)
    
    def test_integration(self) -> bool:
        """Test that modifications don't break existing code"""
        try:
            # Run basic import test
            result = subprocess.run(
                [sys.executable, "-c", "import chonker_snyfter_elegant_v2"],
                capture_output=True,
                text=True
            )
            return result.returncode == 0
        except:
            return False


class ThreeAgentOrchestrator:
    """Orchestrate the three agents for SNYFTER development"""
    
    def __init__(self):
        self.blueprint = SNYFTERBlueprint()
        self.pydantic_agent = PydanticAgent()
        self.instructor_agent = InstructorAgent()
        self.openhands_agent = OpenHandsAgent()
        self.task_results = {}
        
    def execute_autonomous(self):
        """Execute the complete SNYFTER development autonomously"""
        print("🚀 Starting Autonomous SNYFTER Development")
        print("=" * 80)
        
        # Start caffeinate
        start_caffeinate()
        
        try:
            # Phase 1: Create data models
            print("\n📦 Phase 1: Pydantic Data Models")
            result = self.pydantic_agent.create_snyfter_models()
            print(f"  ✅ {result}")
            
            # Phase 2: Generate database schemas
            print("\n🗄️ Phase 2: Database Schema Generation")
            for db_task in self.blueprint.database_tasks:
                schema = self.pydantic_agent.create_database_schema(db_task)
                print(f"  ✅ Generated schema for {db_task.table_name}")
                
                # Save schema
                schema_path = Path(f"snyfter_schema_{db_task.table_name}.sql")
                schema_path.write_text(schema)
            
            # Phase 3: Process development tasks
            print("\n🔧 Phase 3: Development Tasks")
            
            # Group tasks by phase
            phases = [
                ["SNYFTER_001"],
                ["SNYFTER_002"],
                ["SNYFTER_003", "SNYFTER_004"],
                ["SNYFTER_005"]
            ]
            
            for phase_num, phase_tasks in enumerate(phases, 1):
                print(f"\n  Phase {phase_num}:")
                
                for task_id in phase_tasks:
                    task = next((t for t in self.blueprint.tasks if t.id == task_id), None)
                    if not task:
                        continue
                    
                    # Skip AI tasks to avoid prompts
                    if task.type == TaskType.AI_FEATURE:
                        print(f"    ⏭️  Skipping {task.id}: {task.name} (AI feature - manual setup required)")
                        self.task_results[task_id] = "SKIPPED"
                        continue
                    
                    print(f"    🔨 {task.id}: {task.name}")
                    
                    # Generate code with Instructor
                    code = self.instructor_agent.generate_code(task)
                    
                    # Execute with OpenHands
                    success = self.openhands_agent.execute_task(task, code)
                    
                    if success:
                        print(f"    ✅ Completed {task.id}")
                        self.task_results[task_id] = "SUCCESS"
                    else:
                        print(f"    ❌ Failed {task.id}")
                        self.task_results[task_id] = "FAILED"
                    
                    # Test integration
                    if self.openhands_agent.test_integration():
                        print(f"    ✅ Integration test passed")
                    else:
                        print(f"    ⚠️  Integration test failed - manual review needed")
            
            # Generate summary report
            self._generate_summary()
            
        finally:
            stop_caffeinate()
            print("\n✅ Autonomous development completed!")
    
    def _generate_summary(self):
        """Generate development summary"""
        summary = f"""
# SNYFTER Development Summary
Generated: {datetime.now().isoformat()}

## Results
- Total Tasks: {len(self.blueprint.tasks)}
- Successful: {sum(1 for r in self.task_results.values() if r == "SUCCESS")}
- Failed: {sum(1 for r in self.task_results.values() if r == "FAILED")}
- Skipped: {sum(1 for r in self.task_results.values() if r == "SKIPPED")}

## Created Files
{chr(10).join('- ' + f for f in self.openhands_agent.created_files)}

## Modified Files  
{chr(10).join('- ' + f for f in self.openhands_agent.modified_files)}

## Backups
All original files backed up to: {self.openhands_agent.backup_dir}

## Next Steps
1. Review generated code
2. Manually implement AI features (SNYFTER_005)
3. Test SNYFTER mode integration
4. Run comprehensive tests

## Task Details
"""
        for task_id, result in self.task_results.items():
            task = next((t for t in self.blueprint.tasks if t.id == task_id), None)
            if task:
                summary += f"\n### {task_id}: {task.name}\n"
                summary += f"- Status: {result}\n"
                summary += f"- Type: {task.type.value}\n"
                summary += f"- Files: {', '.join(task.files_to_modify)}\n"
        
        # Save summary
        summary_path = Path("snyfter_development_summary.md")
        summary_path.write_text(summary)
        print(f"\n📄 Summary saved to {summary_path}")


if __name__ == "__main__":
    # Ensure we're in virtual environment
    if not hasattr(sys, 'real_prefix') and not (hasattr(sys, 'base_prefix') and sys.base_prefix != sys.prefix):
        print("⚠️  Activating virtual environment...")
        activate_script = Path("venv/bin/activate")
        if activate_script.exists():
            os.system(f"source {activate_script} && python {__file__}")
            sys.exit(0)
        else:
            print("❌ Virtual environment not found!")
            sys.exit(1)
    
    # Run autonomous development
    orchestrator = ThreeAgentOrchestrator()
    orchestrator.execute_autonomous()