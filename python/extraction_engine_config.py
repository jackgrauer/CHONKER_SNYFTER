#!/Users/jack/CHONKER_SNYFTER/venv/bin/python3
"""
Extraction Engine Configuration
Provides compatibility and switching between different extraction engines
"""

import os
import sys
import json
from pathlib import Path
from typing import Dict, Any, Optional

class ExtractionEngineManager:
    """Manages different extraction engines and provides unified interface"""
    
    def __init__(self):
        self.root_dir = Path(__file__).parent.parent
        self.engines = {
            'legacy': 'python/extraction_bridge.py',
            'enhanced': 'python/enhanced_extraction_bridge.py', 
            'pipeline': 'python/extraction_pipeline.py'
        }
        self.default_engine = 'enhanced'
        
    def get_available_engines(self) -> Dict[str, Dict[str, Any]]:
        """Get information about available extraction engines"""
        available = {}
        
        for name, script_path in self.engines.items():
            full_path = self.root_dir / script_path
            if full_path.exists():
                available[name] = {
                    'name': name,
                    'script_path': str(full_path),
                    'description': self._get_engine_description(name),
                    'available': True
                }
            else:
                available[name] = {
                    'name': name,
                    'script_path': str(full_path),
                    'description': self._get_engine_description(name),
                    'available': False
                }
        
        return available
    
    def _get_engine_description(self, engine_name: str) -> str:
        """Get description for an extraction engine"""
        descriptions = {
            'legacy': 'Original environmental lab extraction bridge with domain-specific logic',
            'enhanced': 'Enhanced extraction bridge with improved header recognition and hierarchical structure',
            'pipeline': 'Domain-agnostic 3-stage pipeline with full traceability and debugging'
        }
        return descriptions.get(engine_name, 'Unknown extraction engine')
    
    def get_recommended_engine(self) -> str:
        """Get the recommended extraction engine based on available features"""
        available_engines = self.get_available_engines()
        
        # Prefer enhanced if available
        if available_engines.get('enhanced', {}).get('available', False):
            return 'enhanced'
        
        # Fall back to pipeline
        if available_engines.get('pipeline', {}).get('available', False):
            return 'pipeline'
        
        # Last resort: legacy
        if available_engines.get('legacy', {}).get('available', False):
            return 'legacy'
        
        return None
    
    def create_unified_config(self) -> Dict[str, Any]:
        """Create a unified configuration for all parts of the system"""
        recommended = self.get_recommended_engine()
        available = self.get_available_engines()
        
        config = {
            'default_engine': recommended,
            'available_engines': available,
            'paths': {
                'root_dir': str(self.root_dir),
                'python_dir': str(self.root_dir / 'python'),
                'venv_python': str(self.root_dir / 'venv' / 'bin' / 'python'),
            },
            'engine_selection': {
                'tauri_app': recommended,
                'web_server': recommended,
                'cli_tools': recommended,
                'test_scripts': recommended
            },
            'compatibility': {
                'api_version': '2.0',
                'output_format': 'tauri_compatible',
                'features': self._get_feature_matrix()
            }
        }
        
        return config
    
    def _get_feature_matrix(self) -> Dict[str, Dict[str, bool]]:
        """Get feature compatibility matrix for different engines"""
        return {
            'legacy': {
                'environmental_lab_logic': True,
                'mlx_optimization': True,
                'otsl_format': True,
                'doctags_format': True,
                'hierarchical_headers': False,
                'column_grouping': False,
                'smart_data_types': False,
                'pipeline_traceability': False
            },
            'enhanced': {
                'environmental_lab_logic': False,
                'mlx_optimization': False,
                'otsl_format': False,
                'doctags_format': False,
                'hierarchical_headers': True,
                'column_grouping': True,
                'smart_data_types': True,
                'pipeline_traceability': True
            },
            'pipeline': {
                'environmental_lab_logic': False,
                'mlx_optimization': False,
                'otsl_format': False,
                'doctags_format': False,
                'hierarchical_headers': True,
                'column_grouping': True,
                'smart_data_types': True,
                'pipeline_traceability': True
            }
        }
    
    def save_config(self, config_path: Optional[str] = None) -> Path:
        """Save the unified configuration to a file"""
        if config_path is None:
            config_path = self.root_dir / 'extraction_config.json'
        else:
            config_path = Path(config_path)
        
        config = self.create_unified_config()
        
        with open(config_path, 'w', encoding='utf-8') as f:
            json.dump(config, f, indent=2, ensure_ascii=False)
        
        return config_path
    
    def print_status(self):
        """Print status of all extraction engines"""
        print("🔍 CHONKER Extraction Engine Status")
        print("=" * 50)
        
        available = self.get_available_engines()
        recommended = self.get_recommended_engine()
        
        for name, info in available.items():
            status = "✅" if info['available'] else "❌"
            marker = " (RECOMMENDED)" if name == recommended else ""
            print(f"{status} {name.upper()}: {info['description']}{marker}")
            print(f"   Path: {info['script_path']}")
            print()
        
        if recommended:
            print(f"🚀 Currently using: {recommended.upper()}")
        else:
            print("⚠️  No extraction engines available!")
        
        print("\nFeature Matrix:")
        features = self._get_feature_matrix()
        feature_names = list(next(iter(features.values())).keys())
        
        # Print header
        print(f"{'Feature':<25}", end="")
        for engine in available.keys():
            if available[engine]['available']:
                print(f"{engine.upper():<12}", end="")
        print()
        
        # Print features
        for feature in feature_names:
            print(f"{feature:<25}", end="")
            for engine in available.keys():
                if available[engine]['available']:
                    has_feature = "✅" if features[engine][feature] else "❌"
                    print(f"{has_feature:<12}", end="")
            print()

def main():
    """CLI interface for extraction engine management"""
    import argparse
    
    parser = argparse.ArgumentParser(description='CHONKER Extraction Engine Manager')
    parser.add_argument('--status', action='store_true', help='Show status of all engines')
    parser.add_argument('--config', action='store_true', help='Generate unified configuration')
    parser.add_argument('--output', help='Output file for configuration (default: extraction_config.json)')
    
    args = parser.parse_args()
    
    manager = ExtractionEngineManager()
    
    if args.status:
        manager.print_status()
    elif args.config:
        config_path = manager.save_config(args.output)
        print(f"📄 Configuration saved to: {config_path}")
        print("🔧 Use this config to ensure all app components use the same extraction engine")
    else:
        # Default: show status
        manager.print_status()

if __name__ == '__main__':
    main()
