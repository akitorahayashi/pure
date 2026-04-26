use super::brew::BrewTarget;
use super::category::Category;
use super::docker::DockerTarget;
use super::nodejs::NodejsTarget;
use super::python::PythonTarget;
use super::rust::RustTarget;
use super::target::CleanupTarget;
use super::xcode::XcodeTarget;

const CATEGORY_ORDER: [Category; 6] = [
    Category::Xcode,
    Category::Python,
    Category::Rust,
    Category::Nodejs,
    Category::Brew,
    Category::Docker,
];

pub fn category_order() -> &'static [Category] {
    &CATEGORY_ORDER
}

pub fn categories_for_mode(current: bool) -> Vec<Category> {
    category_order()
        .iter()
        .copied()
        .filter(|category| !current || category.supports_current_mode())
        .collect()
}

pub fn unsupported_for_current(requested: &[Category]) -> Vec<Category> {
    requested.iter().copied().filter(|category| !category.supports_current_mode()).collect()
}

pub fn unique_categories(categories: Vec<Category>) -> Vec<Category> {
    let mut unique = Vec::new();
    for category in categories {
        if !unique.contains(&category) {
            unique.push(category);
        }
    }
    unique
}

pub fn build_targets(categories: &[Category], current: bool) -> Vec<Box<dyn CleanupTarget>> {
    let mut targets: Vec<Box<dyn CleanupTarget>> = Vec::new();

    for category in categories {
        match category {
            Category::Xcode => targets.push(Box::new(XcodeTarget::new(current))),
            Category::Python => targets.push(Box::new(PythonTarget::new())),
            Category::Rust => targets.push(Box::new(RustTarget::new())),
            Category::Nodejs => targets.push(Box::new(NodejsTarget::new())),
            Category::Brew if !current => targets.push(Box::new(BrewTarget::new())),
            Category::Docker if !current => targets.push(Box::new(DockerTarget::new())),
            Category::Brew | Category::Docker => {}
        }
    }

    targets
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn categories_for_current_mode_excludes_system_targets() {
        let categories = categories_for_mode(true);
        assert!(!categories.contains(&Category::Brew));
        assert!(!categories.contains(&Category::Docker));
    }

    #[test]
    fn category_order_is_authoritative_for_default_mode() {
        let categories = categories_for_mode(false);
        assert_eq!(categories, CATEGORY_ORDER);
    }
}
