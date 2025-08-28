#!/usr/bin/env python3
"""
Tag Analysis Tool for spacefeeder

A generic, intelligent tag analysis tool that uses statistical methods
and NLP techniques to identify categorization issues.

Usage:
    python analyze_tags.py [data_dir] [--format json|summary|detailed]
"""

import json
import sys
import argparse
import re
from collections import Counter, defaultdict
from typing import List, Dict, Tuple, Set
from pathlib import Path
import statistics


class TagAnalyzer:
    def __init__(self, items_data: List[Dict]):
        self.items = items_data
        self.tag_counts = Counter()
        self.tag_contexts = defaultdict(list)  # tag -> list of contexts where it appears
        self.total_tags = 0
        self.uncategorized_count = 0
        
        self._process_items()

    def _process_items(self):
        """Extract and count tags from items"""
        for item in self.items:
            tags = item.get('tags', [])
            if not tags:
                self.uncategorized_count += 1
                continue
            
            # Store context for each tag (title + description snippet)
            context = f"{item.get('title', '')} {item.get('safe_description', '')[:100]}"
            
            for tag in tags:
                self.tag_counts[tag] += 1
                self.tag_contexts[tag].append(context)
                self.total_tags += 1

    def analyze(self) -> Dict:
        """Perform comprehensive tag analysis"""
        analysis = {
            'basic_stats': self._basic_statistics(),
            'distribution_analysis': self._distribution_analysis(),
            'quality_issues': self._quality_issues(),
            'semantic_issues': self._semantic_analysis(),
            'recommendations': self._generate_recommendations()
        }
        return analysis

    def _basic_statistics(self) -> Dict:
        """Basic statistical measures"""
        total_items = len(self.items)
        unique_tags = len(self.tag_counts)
        
        return {
            'total_items': total_items,
            'total_tags': self.total_tags,
            'unique_tags': unique_tags,
            'avg_tags_per_item': self.total_tags / total_items if total_items > 0 else 0,
            'uncategorized_items': self.uncategorized_count,
            'uncategorized_percentage': (self.uncategorized_count / total_items * 100) if total_items > 0 else 0
        }

    def _distribution_analysis(self) -> Dict:
        """Analyze tag frequency distribution"""
        if not self.tag_counts:
            return {}
            
        frequencies = list(self.tag_counts.values())
        
        # Calculate distribution metrics
        median_freq = statistics.median(frequencies)
        mean_freq = statistics.mean(frequencies)
        std_freq = statistics.stdev(frequencies) if len(frequencies) > 1 else 0
        
        # Find distribution characteristics
        singleton_tags = [tag for tag, count in self.tag_counts.items() if count == 1]
        rare_tags = [tag for tag, count in self.tag_counts.items() if count <= max(1, mean_freq - 2 * std_freq)]
        
        # Top tags analysis
        most_common = self.tag_counts.most_common(20)
        top_10_percentage = sum(count for _, count in most_common[:10]) / self.total_tags * 100
        
        return {
            'median_frequency': median_freq,
            'mean_frequency': mean_freq,
            'std_frequency': std_freq,
            'singleton_count': len(singleton_tags),
            'singleton_percentage': len(singleton_tags) / len(self.tag_counts) * 100,
            'rare_tags_count': len(rare_tags),
            'top_10_concentration': top_10_percentage,
            'most_common_tags': [{'tag': tag, 'count': count, 'percentage': count/self.total_tags*100} 
                               for tag, count in most_common],
            'singleton_examples': singleton_tags[:20]
        }

    def _quality_issues(self) -> Dict:
        """Identify tag quality issues using heuristics"""
        issues = {
            'long_tags': [],
            'multi_word_tags': [],
            'special_characters': [],
            'potential_proper_nouns': [],
            'very_specific_tags': []
        }
        
        for tag, count in self.tag_counts.items():
            # Long tags (likely too specific)
            if len(tag) > 25:
                issues['long_tags'].append({'tag': tag, 'count': count, 'length': len(tag)})
            
            # Multi-word tags (may need normalization)
            if ' ' in tag and len(tag.split()) > 2:
                issues['multi_word_tags'].append({'tag': tag, 'count': count, 'words': len(tag.split())})
            
            # Tags with special characters
            if re.search(r'[^\w\s\-]', tag):
                issues['special_characters'].append({'tag': tag, 'count': count})
            
            # Potential proper nouns (mixed case, not acronyms)
            if self._is_likely_proper_noun(tag):
                issues['potential_proper_nouns'].append({'tag': tag, 'count': count})
            
            # Very specific tags (appear rarely and are long/complex)
            if count <= 2 and (len(tag) > 15 or ' ' in tag):
                issues['very_specific_tags'].append({'tag': tag, 'count': count})
        
        # Sort by count (descending) and limit examples
        for issue_type, tag_list in issues.items():
            issues[issue_type] = sorted(tag_list, key=lambda x: x['count'], reverse=True)[:15]
        
        return issues

    def _is_likely_proper_noun(self, tag: str) -> bool:
        """Heuristic to detect proper nouns"""
        if len(tag) <= 3:  # Skip acronyms
            return False
            
        # Check for mixed case (not all upper or all lower)
        has_upper = any(c.isupper() for c in tag)
        has_lower = any(c.islower() for c in tag)
        
        if not (has_upper and has_lower):
            return False
        
        # Common proper noun patterns
        if re.match(r'^[A-Z][a-z]+(?:\s[A-Z][a-z]+)*$', tag):
            return True
            
        # Contains typical proper noun indicators
        proper_indicators = ['-van-', '-de-', '-von-', '-el-', '-la-', '-du-']
        return any(indicator in tag.lower() for indicator in proper_indicators)

    def _semantic_analysis(self) -> Dict:
        """Analyze semantic patterns and relationships"""
        # Find potential duplicates and similar tags
        similar_groups = self._find_similar_tags()
        
        # Analyze tag co-occurrence patterns
        cooccurrence = self._analyze_cooccurrence()
        
        # Detect potential category hierarchies
        hierarchies = self._detect_hierarchies()
        
        return {
            'similar_tag_groups': similar_groups,
            'high_cooccurrence_pairs': cooccurrence,
            'potential_hierarchies': hierarchies
        }

    def _find_similar_tags(self) -> List[Dict]:
        """Find tags that might be duplicates or very similar"""
        similar_groups = []
        tags = list(self.tag_counts.keys())
        
        for i, tag1 in enumerate(tags):
            for tag2 in tags[i+1:]:
                similarity_score = self._tag_similarity(tag1, tag2)
                if similarity_score > 0.7:  # High similarity threshold
                    similar_groups.append({
                        'tags': [tag1, tag2],
                        'counts': [self.tag_counts[tag1], self.tag_counts[tag2]],
                        'similarity': similarity_score
                    })
        
        return sorted(similar_groups, key=lambda x: x['similarity'], reverse=True)[:10]

    def _tag_similarity(self, tag1: str, tag2: str) -> float:
        """Calculate similarity between two tags"""
        # Simple similarity measures
        
        # Exact substring match
        if tag1 in tag2 or tag2 in tag1:
            return 0.9
            
        # Common word-based similarity
        words1 = set(tag1.lower().replace('-', ' ').split())
        words2 = set(tag2.lower().replace('-', ' ').split())
        
        if words1 and words2:
            intersection = len(words1 & words2)
            union = len(words1 | words2)
            jaccard = intersection / union if union > 0 else 0
            return jaccard
            
        return 0

    def _analyze_cooccurrence(self) -> List[Dict]:
        """Analyze which tags frequently appear together"""
        cooccurrence = defaultdict(int)
        
        for item in self.items:
            tags = item.get('tags', [])
            if len(tags) > 1:
                for i, tag1 in enumerate(tags):
                    for tag2 in tags[i+1:]:
                        pair = tuple(sorted([tag1, tag2]))
                        cooccurrence[pair] += 1
        
        # Filter and sort by frequency
        significant_pairs = [
            {
                'tags': list(pair), 
                'cooccurrence_count': count,
                'tag1_total': self.tag_counts[pair[0]],
                'tag2_total': self.tag_counts[pair[1]],
                'cooccurrence_rate': count / min(self.tag_counts[pair[0]], self.tag_counts[pair[1]])
            }
            for pair, count in cooccurrence.items() 
            if count >= 3  # Minimum co-occurrence threshold
        ]
        
        return sorted(significant_pairs, key=lambda x: x['cooccurrence_rate'], reverse=True)[:15]

    def _detect_hierarchies(self) -> List[Dict]:
        """Detect potential tag hierarchies (general -> specific)"""
        hierarchies = []
        
        # Look for tags where one might be a specialization of another
        for general_tag, general_count in self.tag_counts.items():
            potential_specific = []
            
            for specific_tag, specific_count in self.tag_counts.items():
                if general_tag != specific_tag:
                    # Check if specific_tag could be a specialization of general_tag
                    if (general_tag.lower() in specific_tag.lower() or 
                        any(word in specific_tag.lower() for word in general_tag.lower().split()) and
                        specific_count < general_count * 0.5):  # Specific should be less common
                        potential_specific.append({'tag': specific_tag, 'count': specific_count})
            
            if potential_specific:
                hierarchies.append({
                    'general_tag': general_tag,
                    'general_count': general_count,
                    'specific_tags': sorted(potential_specific, key=lambda x: x['count'], reverse=True)[:5]
                })
        
        return sorted(hierarchies, key=lambda x: len(x['specific_tags']), reverse=True)[:10]

    def _generate_recommendations(self) -> List[str]:
        """Generate actionable recommendations based on analysis"""
        recommendations = []
        
        stats = self._basic_statistics()
        dist = self._distribution_analysis()
        quality = self._quality_issues()
        
        # Uncategorized items
        if stats['uncategorized_percentage'] > 15:
            recommendations.append(f"High uncategorized rate ({stats['uncategorized_percentage']:.1f}%): "
                                 "Review and expand categorization rules")
        
        # Singleton tags
        if dist.get('singleton_percentage', 0) > 40:
            recommendations.append(f"High singleton rate ({dist['singleton_percentage']:.1f}%): "
                                 "Consider filtering tags that appear only once")
        
        # Tag distribution concentration
        concentration = dist.get('top_10_concentration', 0)
        if concentration > 80:
            recommendations.append("Very concentrated tag distribution: Consider expanding categorization diversity")
        elif concentration < 25:
            recommendations.append("Very dispersed tag distribution: Consider consolidating related tags")
        
        # Quality issues
        if quality['long_tags']:
            recommendations.append(f"{len(quality['long_tags'])} very long tags found: "
                                 "Consider aliases for normalization")
        
        if quality['multi_word_tags']:
            recommendations.append(f"{len(quality['multi_word_tags'])} complex multi-word tags found: "
                                 "Consider hyphenation or aliases")
        
        if quality['potential_proper_nouns']:
            recommendations.append(f"{len(quality['potential_proper_nouns'])} potential proper noun tags found: "
                                 "Consider broader category aliases")
        
        # Average tags per item
        if stats['avg_tags_per_item'] < 1.5:
            recommendations.append("Low average tags per item: Consider expanding categorization rules")
        elif stats['avg_tags_per_item'] > 4:
            recommendations.append("High average tags per item: Consider consolidating or filtering")
        
        if not recommendations:
            recommendations.append("Tag distribution appears healthy - no major issues detected")
        
        return recommendations


def load_data(data_dir: Path) -> List[Dict]:
    """Load itemData.json from the specified directory"""
    item_data_path = data_dir / "content" / "data" / "itemData.json"
    
    if not item_data_path.exists():
        raise FileNotFoundError(f"Could not find {item_data_path}")
    
    with open(item_data_path, 'r') as f:
        return json.load(f)


def print_summary(analysis: Dict):
    """Print a summary analysis"""
    stats = analysis['basic_stats']
    dist = analysis['distribution_analysis']
    
    print("📊 Tag Analysis Summary")
    print("=" * 40)
    print(f"Total articles: {stats['total_items']}")
    print(f"Total tags assigned: {stats['total_tags']}")
    print(f"Unique tags: {stats['unique_tags']}")
    print(f"Average tags per article: {stats['avg_tags_per_item']:.1f}")
    print(f"Uncategorized articles: {stats['uncategorized_items']} ({stats['uncategorized_percentage']:.1f}%)")
    print()
    
    if dist:
        print(f"🔸 Singleton tags: {dist['singleton_count']} ({dist['singleton_percentage']:.1f}%)")
        print(f"📊 Top 10 tag concentration: {dist['top_10_concentration']:.1f}%")
        print()
    
    # Quality issues
    quality = analysis['quality_issues']
    issues_found = sum(len(issues) for issues in quality.values())
    if issues_found > 0:
        print("⚠️  Potential Issues:")
        for issue_type, issues in quality.items():
            if issues:
                issue_name = issue_type.replace('_', ' ').title()
                print(f"  • {issue_name}: {len(issues)} instances")
        print()
    
    # Top tags
    if dist and dist['most_common_tags']:
        print("📈 Top 10 Tags:")
        for i, tag_info in enumerate(dist['most_common_tags'][:10], 1):
            print(f"  {i}. {tag_info['tag']} ({tag_info['count']} articles, {tag_info['percentage']:.1f}%)")
        print()
    
    # Recommendations
    recommendations = analysis['recommendations']
    if recommendations:
        print("💡 Recommendations:")
        for rec in recommendations:
            print(f"  • {rec}")


def print_detailed(analysis: Dict):
    """Print detailed analysis"""
    print_summary(analysis)
    print("\n" + "=" * 60)
    
    quality = analysis['quality_issues']
    
    # Detailed quality issues
    for issue_type, issues in quality.items():
        if issues:
            issue_name = issue_type.replace('_', ' ').title()
            print(f"\n🔍 {issue_name}:")
            for issue in issues[:10]:
                print(f"  • '{issue['tag']}' (appears {issue['count']} times)")
    
    # Semantic analysis
    semantic = analysis['semantic_analysis']
    
    if semantic['similar_tag_groups']:
        print("\n🔗 Similar Tag Groups:")
        for group in semantic['similar_tag_groups'][:5]:
            tags_str = " / ".join(group['tags'])
            counts_str = " / ".join(map(str, group['counts']))
            print(f"  • {tags_str} (counts: {counts_str}, similarity: {group['similarity']:.2f})")
    
    if semantic['high_cooccurrence_pairs']:
        print("\n🤝 Frequently Co-occurring Tags:")
        for pair in semantic['high_cooccurrence_pairs'][:5]:
            tags_str = " + ".join(pair['tags'])
            rate = pair['cooccurrence_rate'] * 100
            print(f"  • {tags_str} (co-occur {pair['cooccurrence_count']} times, {rate:.1f}% rate)")


def main():
    parser = argparse.ArgumentParser(description="Analyze tag quality and distribution")
    parser.add_argument("data_dir", nargs="?", default=".", help="Data directory (default: current directory)")
    parser.add_argument("--format", choices=["summary", "detailed", "json"], default="summary",
                       help="Output format")
    
    args = parser.parse_args()
    
    try:
        # Load data
        data_dir = Path(args.data_dir)
        items = load_data(data_dir)
        
        # Analyze
        analyzer = TagAnalyzer(items)
        analysis = analyzer.analyze()
        
        # Output
        if args.format == "json":
            print(json.dumps(analysis, indent=2))
        elif args.format == "detailed":
            print_detailed(analysis)
        else:
            print_summary(analysis)
            
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()